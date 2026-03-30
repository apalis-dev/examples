use apalis::prelude::{Data, WorkerBuilder};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{storage::Storage, types::ReminderNotification};

mod error;
mod storage;
mod types;

async fn notify_reminder(
    input: ReminderNotification,
    app: Data<AppHandle>,
    storage: Data<Storage>,
) -> Result<(), error::Error> {
    let reminder = storage
        .get_by_id(input.reminder_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    app.notification()
        .builder()
        .title(reminder.title)
        .body(reminder.reminder_text)
        .show()?;
    println!("Handled {:?}", input);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let app_handle = app.handle();
            let handle_clone = app_handle.clone();

            let job_store = tauri::async_runtime::block_on(async move {
                Storage::new(handle_clone)
                    .await
                    .expect("Failed to initialize database")
            });

            let worker = WorkerBuilder::new("fancy-worker")
                .backend(job_store.tasks.clone())
                .data(app_handle.clone())
                .data(job_store.clone())
                .build(notify_reminder)
                .run();

            tauri::async_runtime::spawn(worker);

            app.manage(job_store);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            storage::add_reminder,
            storage::update_reminder,
            storage::delete_reminder,
            storage::fetch_reminders
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
