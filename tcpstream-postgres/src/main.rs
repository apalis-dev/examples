use apalis::prelude::*;
use apalis_postgres::*;
use bytes::BytesMut;
use futures::Stream;
use futures::StreamExt;
use futures::TryFutureExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::ready;
use std::io::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio_util::codec::Decoder;
use tokio_util::codec::FramedRead;

#[derive(Serialize, Deserialize, Debug)]
struct MyMsg {
    id: u64,
}

async fn handle_task(item: MyMsg, wrk: WorkerContext) -> Result<(), BoxDynError> {
    println!("Received {item:?}");
    Ok(())
}

#[derive(Debug)]
pub enum TaskListenerError {
    Io(Error),
    Json(serde_json::Error),
    FrameTooLarge,
    TooManyConnections,
}

pub struct TaskListenerConfig {
    pub max_connections: usize,
    pub max_frame_bytes: usize,
    pub channel_capacity: usize,
}

struct NdjsonDecoder {
    max_len: usize,
}

impl Decoder for NdjsonDecoder {
    type Item = BytesMut;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(pos) = buf.iter().position(|b| *b == b'\n') {
            if pos > self.max_len {
                return Err(Error::new(ErrorKind::InvalidData, "frame too large"));
            }

            let mut frame = buf.split_to(pos + 1);
            frame.truncate(pos);
            return Ok(Some(frame));
        }

        if buf.len() > self.max_len {
            return Err(Error::new(ErrorKind::InvalidData, "frame too large"));
        }

        Ok(None)
    }
}

pub struct TaskListenerStream<T> {
    receiver: mpsc::Receiver<Result<T, TaskListenerError>>,
}

async fn run_accept_loop<T>(
    listener: TcpListener,
    sender: mpsc::Sender<Result<T, TaskListenerError>>,
    semaphore: Arc<Semaphore>,
    config: TaskListenerConfig,
) where
    T: DeserializeOwned + Send + 'static,
{
    loop {
        match listener.accept().await {
            Ok((socket, _)) => match semaphore.clone().try_acquire_owned() {
                Ok(permit) => {
                    let tx = sender.clone();
                    tokio::spawn(async move {
                        handle_connection::<T>(socket, tx, config.max_frame_bytes).await;
                        drop(permit);
                    });
                }
                Err(_) => {
                    let _ = sender
                        .send(Err(TaskListenerError::TooManyConnections))
                        .await;
                }
            },
            Err(e) => {
                let _ = sender.send(Err(TaskListenerError::Io(e))).await;
            }
        }
    }
}

async fn handle_connection<T>(
    socket: tokio::net::TcpStream,
    sender: mpsc::Sender<Result<T, TaskListenerError>>,
    max_frame: usize,
) where
    T: DeserializeOwned + Send + 'static,
{
    let mut framed = FramedRead::new(socket, NdjsonDecoder { max_len: max_frame });

    while let Some(frame) = framed.next().await {
        match frame {
            Ok(bytes) => {
                let parsed = serde_json::from_slice::<T>(&bytes).map_err(TaskListenerError::Json);

                if sender.send(parsed).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                let _ = sender.send(Err(TaskListenerError::Io(e))).await;
                break;
            }
        }
    }
}

impl<T> TaskListenerStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    pub async fn bind(addr: &str, config: TaskListenerConfig) -> Result<Self, Error> {
        let listener = TcpListener::bind(addr).await?;
        let (tx, rx) = mpsc::channel(config.channel_capacity);

        let semaphore = Arc::new(Semaphore::new(config.max_connections));

        tokio::spawn(run_accept_loop::<T>(listener, tx, semaphore, config));

        Ok(Self { receiver: rx })
    }
}

impl<T> Stream for TaskListenerStream<T> {
    type Item = Result<T, TaskListenerError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_recv(cx)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db).await?;

    PostgresStorage::setup(&pool).await.unwrap();
    let mut backend = PostgresStorage::new_with_notify(&pool, &Config::default());

    let mut stream = TaskListenerStream::<MyMsg>::bind(
        "127.0.0.1:5000",
        TaskListenerConfig {
            max_connections: 100,
            max_frame_bytes: 8 * 1024,
            channel_capacity: 1024,
        },
    )
    .await?
    .filter_map(|res| ready(res.ok()));

    let worker = WorkerBuilder::new("worker-1")
        .backend(backend.clone())
        .build(handle_task)
        .run()
        .map_err(|e| Error::new(ErrorKind::Interrupted, e));

    let handler = backend
        .push_stream(&mut stream)
        .map_err(|e| Error::new(ErrorKind::BrokenPipe, e));

    tokio::try_join!(worker, handler)?;

    Ok(())
}
