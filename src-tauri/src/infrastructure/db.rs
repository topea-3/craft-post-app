use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqlitePoolOptions};
use sqlx::SqlitePool;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};

pub const DB_FILE_NAME: &str = "craft_post.db";
pub const INCOMING_FILE_NAME: &str = "craft_post.db.incoming";
pub const PRE_RESTORE_FILE_NAME: &str = "craft_post.db.pre-restore";
pub const SNAPSHOT_FILE_NAME: &str = "craft_post.backup-snapshot.db";
pub const VERIFY_FILE_NAME: &str = "craft_post.db.verify";
pub const DB_BUSY_MESSAGE: &str =
  "他の処理がデータベースを使用中です。しばらくしてから再度実行してください。";

const READ_LOCK_WAIT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct AppDb {
  inner: Arc<RwLock<SqlitePool>>,
  verify: Arc<Mutex<()>>,
  db_path: std::sync::Arc<PathBuf>,
}

pub struct AppDbReadGuard<'a> {
  _guard: RwLockReadGuard<'a, SqlitePool>,
}

impl Deref for AppDbReadGuard<'_> {
  type Target = SqlitePool;

  fn deref(&self) -> &Self::Target {
    &self._guard
  }
}

impl AppDb {
  pub fn new(pool: SqlitePool, db_path: PathBuf) -> Self {
    Self {
      inner: Arc::new(RwLock::new(pool)),
      verify: Arc::new(Mutex::new(())),
      db_path: std::sync::Arc::new(db_path),
    }
  }

  pub fn db_path(&self) -> &Path {
    self.db_path.as_path()
  }

  pub fn pool_lock(&self) -> Arc<RwLock<SqlitePool>> {
    Arc::clone(&self.inner)
  }

  pub fn verify_lock(&self) -> Arc<Mutex<()>> {
    Arc::clone(&self.verify)
  }

  pub async fn read(&self) -> Result<AppDbReadGuard<'_>, String> {
    match tokio::time::timeout(READ_LOCK_WAIT, self.inner.read()).await {
      Ok(guard) => Ok(AppDbReadGuard { _guard: guard }),
      Err(_) => Err(DB_BUSY_MESSAGE.to_string()),
    }
  }
}

pub fn sibling_path(db_path: &Path, name: &str) -> PathBuf {
  match db_path.parent() {
    Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
    _ => PathBuf::from(name),
  }
}

pub fn latest_schema_version() -> i64 {
  sqlx::migrate!()
    .iter()
    .map(|migration| migration.version)
    .max()
    .unwrap_or(0)
}

/// アプリのデータディレクトリ配下に SQLite ファイルを配置するためのパスを解決する。
pub fn resolve_db_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  let resolver = app.path();
  let path = resolver.resolve(DB_FILE_NAME, BaseDirectory::AppData)?;

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  Ok(path)
}

pub async fn init_app_db(app: &AppHandle) -> Result<AppDb, sqlx::Error> {
  let db_path = resolve_db_path(app).map_err(|error| sqlx::Error::Configuration(Box::new(error)))?;
  log::debug!("SQLite DB path: {}", db_path.to_string_lossy());
  recover_startup(&db_path).await?;
  let create_if_missing = !db_path.exists();
  let pool = open_pool(&db_path, create_if_missing).await?;
  Ok(AppDb::new(pool, db_path))
}

#[cfg(test)]
pub async fn open_app_db_at(db_path: PathBuf) -> Result<AppDb, sqlx::Error> {
  if let Some(parent) = db_path.parent() {
    fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
  }
  recover_startup(&db_path).await?;
  let create_if_missing = !db_path.exists();
  let pool = open_pool(&db_path, create_if_missing).await?;
  Ok(AppDb::new(pool, db_path))
}

pub async fn open_pool(path: &Path, create_if_missing: bool) -> Result<SqlitePool, sqlx::Error> {
  let options = SqliteConnectOptions::new()
    .filename(path)
    .create_if_missing(create_if_missing);
  let pool = SqlitePool::connect_with(options).await?;
  sqlx::migrate!().run(&pool).await?;
  Ok(pool)
}

pub async fn open_incoming_pool(path: &Path) -> Result<SqlitePool, sqlx::Error> {
  let options = SqliteConnectOptions::new()
    .filename(path)
    .create_if_missing(false)
    .pragma("trusted_schema", "OFF");
  let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .connect_with(options)
    .await?;
  {
    let mut connection = pool.acquire().await?;
    if let Err(error) = harden_connection(&mut connection).await {
      drop(connection);
      pool.close().await;
      return Err(error);
    }
  }
  if let Err(error) = sqlx::migrate!().run(&pool).await {
    pool.close().await;
    return Err(sqlx::Error::from(error));
  }
  Ok(pool)
}

async fn harden_connection(connection: &mut SqliteConnection) -> Result<(), sqlx::Error> {
  let mut handle = connection.lock_handle().await?;
  let raw = handle.as_raw_handle().as_ptr();
  let defensive = unsafe {
    libsqlite3_sys::sqlite3_db_config(
      raw,
      libsqlite3_sys::SQLITE_DBCONFIG_DEFENSIVE,
      1,
      std::ptr::null_mut::<i32>(),
    )
  };
  let extensions = unsafe {
    libsqlite3_sys::sqlite3_db_config(
      raw,
      libsqlite3_sys::SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION,
      0,
      std::ptr::null_mut::<i32>(),
    )
  };
  if defensive != libsqlite3_sys::SQLITE_OK || extensions != libsqlite3_sys::SQLITE_OK {
    return Err(sqlx::Error::Configuration(Box::new(std::io::Error::other(
      "failed to harden sqlite connection",
    ))));
  }
  Ok(())
}

async fn recover_startup(db_path: &Path) -> Result<(), sqlx::Error> {
  let pre_restore = sibling_path(db_path, PRE_RESTORE_FILE_NAME);
  match (db_path.exists(), pre_restore.exists()) {
    (false, false) => {}
    (false, true) => {
      fs::rename(&pre_restore, db_path).map_err(sqlx::Error::Io)?;
    }
    (true, false) => {}
    (true, true) => match open_pool(db_path, false).await {
      Ok(pool) => {
        pool.close().await;
        fs::remove_file(&pre_restore).map_err(sqlx::Error::Io)?;
      }
      Err(_) => {
        fs::remove_file(db_path).map_err(sqlx::Error::Io)?;
        fs::rename(&pre_restore, db_path).map_err(sqlx::Error::Io)?;
        let pool = open_pool(db_path, false).await?;
        pool.close().await;
      }
    },
  }

  for name in [INCOMING_FILE_NAME, SNAPSHOT_FILE_NAME, VERIFY_FILE_NAME] {
    let path = sibling_path(db_path, name);
    if path.exists() {
      fs::remove_file(path).map_err(sqlx::Error::Io)?;
    }
  }
  Ok(())
}
