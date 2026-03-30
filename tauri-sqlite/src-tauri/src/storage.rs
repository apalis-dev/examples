use std::{env, fs};

use apalis::prelude::{TaskBuilder, TaskSink};
use apalis_codec::json::JsonCodec;
use apalis_sqlite::{
    Config, HookCallbackListener, SqliteConnectOptions, SqlitePool, SqliteStorage,
};
use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::{error::Error, types::*};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Storage {
    pub tasks: SqliteStorage<ReminderNotification, JsonCodec<Vec<u8>>, HookCallbackListener>,
    pub pool: SqlitePool,
}

impl Storage {
    pub async fn new(app_handle: AppHandle) -> Result<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");

        // Ensure the app directory exists
        fs::create_dir_all(&app_dir)?;

        let db_path = app_dir.join("reminders.db");

        // Set the DATABASE_URL environment variable to point to this SQLite file
        env::set_var("DATABASE_URL", format!("sqlite://{}", db_path.display()));

        println!("-----------------------------------------------");
        println!("Initializing database at: {:?}", db_path);
        println!("-----------------------------------------------");

        // Create the connection options
        let connection_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        // Create and initialize the database pool
        let pool = SqlitePool::connect_with(connection_options).await?;

        SqliteStorage::setup(&pool).await?;
        Self::setup(&pool).await?;

        Ok(Self {
            tasks: SqliteStorage::new_with_callback(
                &env::var("DATABASE_URL").unwrap(),
                &Config::default(),
            ),
            pool,
        })
    }

    pub async fn setup(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                scheduled_time TEXT NOT NULL,
                reminder_text TEXT NOT NULL,
                note TEXT NOT NULL,
                is_complete INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
        "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn create(&self, reminder: &NewReminder) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO reminders (
                title, scheduled_time, reminder_text, note, is_complete, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reminder.title)
        .bind(&reminder.scheduled_time)
        .bind(&reminder.reminder_text)
        .bind(&reminder.note)
        .bind(false)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<Reminder>> {
        let reminder = sqlx::query_as::<_, Reminder>(
            r#"
            SELECT 
                id,
                title,
                scheduled_time,
                reminder_text,
                note,
                is_complete,
                created_at
            FROM reminders
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(reminder)
    }

    pub async fn list(&self) -> Result<Vec<Reminder>> {
        let reminders = sqlx::query_as::<_, Reminder>(
            r#"
            SELECT 
                id,
                title,
                scheduled_time,
                reminder_text,
                note,
                is_complete,
                created_at
            FROM reminders
            ORDER BY scheduled_time ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reminders)
    }

    pub async fn update(&self, id: i64, update: UpdateReminder) -> Result<()> {
        let existing = self.get_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)?;

        sqlx::query(
            r#"
            UPDATE reminders
            SET 
                title = ?,
                scheduled_time = ?,
                reminder_text = ?,
                note = ?,
                is_complete = ?
            WHERE id = ?
            "#,
        )
        .bind(update.title.unwrap_or(existing.title))
        .bind(update.scheduled_time.unwrap_or(existing.scheduled_time))
        .bind(update.reminder_text.unwrap_or(existing.reminder_text))
        .bind(update.note.unwrap_or(existing.note))
        .bind(update.is_complete.unwrap_or(existing.is_complete))
        .bind(id)
        .execute(&self.pool)
        .await?;

        let update_job = r#"
            UPDATE Jobs
                SET
                    run_at = (
                        SELECT strftime('%s', scheduled_time)
                        FROM Reminders
                        WHERE id = ?
                    ),
                    status = CASE
                        WHEN (
                            SELECT strftime('%s', scheduled_time)
                            FROM Reminders
                            WHERE id = ?
                        ) <= strftime('%s', 'now')
                        THEN 'Killed'
                        ELSE 'Pending'
                    END
                WHERE json_extract(job, '$.reminderId') = ?;
        "#;

        sqlx::query(update_job)
            .bind(id)
            .bind(id)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM reminders
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(unused)]
    pub async fn mark_complete(&self, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE reminders
            SET is_complete = 1
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[tauri::command]
pub async fn add_reminder(reminder: NewReminder, state: tauri::State<'_, Storage>) -> Result<i64> {
    let reminder_id = state.create(&reminder).await?;

    let mut store = state.inner().tasks.clone();

    let new_task = TaskBuilder::new(ReminderNotification { reminder_id })
        .run_at_time(reminder.scheduled_time.try_into().unwrap())
        .build();

    store
        .push_task(new_task)
        .await
        .map_err(|e| Error::TaskSink(e.into()))?;
    Ok(reminder_id)
}

#[tauri::command]
pub async fn update_reminder(
    id: i64,
    reminder: UpdateReminder,
    state: tauri::State<'_, Storage>,
) -> Result<()> {
    state.update(id, reminder).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_reminder(id: i64, state: tauri::State<'_, Storage>) -> Result<()> {
    state.delete(id).await?;
    Ok(())
}

#[tauri::command]
pub async fn fetch_reminders(state: tauri::State<'_, Storage>) -> Result<Vec<Reminder>> {
    Ok(state.list().await?)
}
