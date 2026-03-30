use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub scheduled_time: DateTime<Utc>,
    pub reminder_text: String,
    pub note: String,
    pub is_complete: bool,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReminder {
    pub title: String,
    pub scheduled_time: DateTime<Utc>,
    pub reminder_text: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct UpdateReminder {
    pub title: Option<String>,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub reminder_text: Option<String>,
    pub note: Option<String>,
    pub is_complete: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderNotification {
    pub reminder_id: i64,
}
