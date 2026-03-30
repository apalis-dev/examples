use apalis::prelude::BoxDynError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not push task: {0}")]
    TaskSink(BoxDynError),
    #[error("std::io::Error: `{0}`")]
    IO(#[from] std::io::Error),
    #[error("SqlxError: `{0}`")]
    Sqlx(#[from] sqlx::Error),

    #[error("NotificationError: `{0}`")]
    Notification(#[from] tauri_plugin_notification::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
