use std::fs;
use std::path::PathBuf;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

const DB_FILE_NAME: &str = "craft_post.db";

/// アプリのデータディレクトリ配下に SQLite ファイルを配置するためのパスを解決する。
pub fn resolve_db_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  let resolver = app.path();
  let path = resolver.resolve(DB_FILE_NAME, BaseDirectory::AppData)?;

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  Ok(path)
}

/// SQLite プールを初期化する。
///
/// - まだファイルが無ければ自動作成される。
/// - プール作成後にマイグレーションを適用する。
pub async fn init_pool(app: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
  let db_path = resolve_db_path(app).map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
  let options = SqliteConnectOptions::new()
    .filename(&db_path)
    .create_if_missing(true);

  let pool = SqlitePool::connect_with(options).await?;

  // マイグレーション適用（クレート直下の `migrations/` ディレクトリを前提とする）
  sqlx::migrate!().run(&pool).await?;

  Ok(pool)
}

