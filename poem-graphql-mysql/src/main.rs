use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use apalis::layers::retry::RetryPolicy;
use apalis::prelude::*;
use apalis_board_api::sse::{TracingBroadcaster, TracingSubscriber};
use apalis_codec::json::JsonCodec;
use apalis_mysql::fetcher::MySqlFetcher;
use apalis_mysql::{MySqlPool, MySqlStorage};
use async_graphql::InputObject;
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, Object, Result, Schema, Subscription, extensions::OpenTelemetry};
use async_graphql_poem::{GraphQL, GraphQLSubscription};
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use poem::web::Html;
use poem::{EndpointExt, Route, Server, listener::TcpListener, post};
use poem::{IntoResponse, get, handler};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

type Storage = MySqlStorage<Job, JsonCodec<Vec<u8>>, MySqlFetcher>;

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
struct Job {
    id: String,
}

struct Query;

#[Object]
impl Query {
    async fn get_by_id(&self, ctx: &Context<'_>, id: String) -> Result<serde_json::Value> {
        let store: &Storage = ctx.data().unwrap();
        let mut store = store.clone();
        let job = store
            .fetch_by_id(&TaskId::from_str(&id).unwrap())
            .await?
            .unwrap();
        Ok(serde_json::to_value(job).unwrap())
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn push_job(&self, ctx: &Context<'_>, job: Job) -> Result<bool> {
        let store: &Storage = ctx.data().unwrap();
        let mut store = store.clone();
        store.push(job).await?;
        Ok(true)
    }
}

struct Subscription;

#[Subscription]
impl Subscription {
    async fn events(&self, ctx: &Context<'_>) -> impl Stream<Item = serde_json::Value> {
        let broadcaster: &Arc<Mutex<TracingBroadcaster>> = ctx.data().unwrap();
        let client = broadcaster.lock().unwrap().new_client();
        client.map(|s| serde_json::to_value(s.unwrap()).unwrap())
    }
}

async fn task_handler(_msg: Job) -> Result<(), BoxDynError> {
    tracing::info!("Executing task");
    tokio::time::sleep(Duration::from_secs(1)).await;
    tracing::info!("Started task");
    tokio::time::sleep(Duration::from_secs(1)).await;
    tracing::info!("Completed task");
    Ok(())
}

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // build OTEL provider
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("poem-opentelemetry-apalis");
    let opentelemetry_extension = OpenTelemetry::new(tracer);

    let broadcaster: Arc<Mutex<TracingBroadcaster>> = TracingBroadcaster::create();

    let tracing_subscriber = TracingSubscriber::new(&broadcaster);
    let tracing_layer = tracing_subscriber
        .layer()
        .with_filter(EnvFilter::builder().parse("debug").unwrap());

    let stdio_layer =
        tracing_subscriber::fmt::layer().with_filter(EnvFilter::builder().parse("debug").unwrap());

    tracing_subscriber::registry()
        .with(stdio_layer)
        .with(tracing_layer)
        .init();

    let pool = MySqlPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    MySqlStorage::setup(&pool).await.unwrap();
    let backend = MySqlStorage::new(&pool);

    let worker = WorkerBuilder::new("ntfy-banana")
        .backend(backend.clone())
        .enable_tracing()
        .retry(RetryPolicy::retries(3))
        .concurrency(4)
        .build(task_handler)
        .run_until(tokio::signal::ctrl_c())
        .map_err(std::io::Error::other);

    // build GraphQL schema
    let schema = Schema::build(Query, Mutation, Subscription)
        .extension(opentelemetry_extension)
        .data(broadcaster)
        .data(backend)
        .finish();

    let app = Route::new()
        .at("/", get(graphiql))
        .at("/graphql", post(GraphQL::new(schema.clone())))
        .at("/ws", get(GraphQLSubscription::new(schema.clone())))
        .data(schema);

    println!("Poem server started at http://0.0.0.0:3000");
    println!("GraphQL endpoint at http://0.0.0.0:3000/graphql");

    let http = Server::new(TcpListener::bind("0.0.0.0:3000")).run_with_graceful_shutdown(
        app,
        tokio::signal::ctrl_c().map(|_| ()),
        None,
    );
    futures::future::try_join(http, worker).await?;
    Ok(())
}
