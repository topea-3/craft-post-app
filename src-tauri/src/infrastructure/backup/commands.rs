use std::path::PathBuf;

use tauri::State;

use crate::infrastructure::db::AppDb;

use super::container::BackupInspection;
use super::ops;

#[tauri::command]
pub async fn create_backup(
  db: State<'_, AppDb>,
  destination_path: String,
  password: Option<String>,
) -> Result<BackupInspection, String> {
  ops::create_backup(db.inner(), PathBuf::from(destination_path), password)
    .await
    .map_err(|error| {
      log::error!("create_backup failed: {}", error.code);
      error.to_json()
    })
}

#[tauri::command]
pub async fn inspect_backup(
  db: State<'_, AppDb>,
  source_path: String,
  password: Option<String>,
) -> Result<BackupInspection, String> {
  ops::inspect_backup(db.inner(), PathBuf::from(source_path), password)
    .await
    .map_err(|error| {
      log::error!("inspect_backup failed: {}", error.code);
      error.to_json()
    })
}

#[tauri::command]
pub async fn restore_backup(
  db: State<'_, AppDb>,
  source_path: String,
  password: Option<String>,
) -> Result<BackupInspection, String> {
  let db = db.inner().clone();
  ops::restore_backup(db, PathBuf::from(source_path), password)
    .await
    .map_err(|error| {
      log::error!("restore_backup failed: {}", error.code);
      error.to_json()
    })
}
