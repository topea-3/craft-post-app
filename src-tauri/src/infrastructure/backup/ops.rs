use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Local;
use sqlx::SqlitePool;
use tokio::sync::OwnedRwLockWriteGuard;
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, Zeroizing};

use crate::infrastructure::db::{
  open_incoming_pool, open_pool, sibling_path, AppDb, INCOMING_FILE_NAME, PRE_RESTORE_FILE_NAME,
  SNAPSHOT_FILE_NAME, VERIFY_FILE_NAME,
};

use super::container::{self, BackupInspection, Manifest, MAX_CONTAINER_BYTES};
use super::error::BackupError;
use super::paths::{self, partial_path};
use super::sqlite_raw;

const WRITE_LOCK_WAIT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreFailpoint {
  AfterCloseBeforeRename,
  AfterSwapBeforeOpen,
}

pub fn normalize_password(password: Option<String>) -> Result<Option<Zeroizing<String>>, BackupError> {
  let Some(mut raw) = password else {
    return Ok(None);
  };
  let normalized: String = raw.nfc().collect();
  raw.zeroize();
  let scalars = normalized.chars().count();
  if !(8..=128).contains(&scalars) {
    let mut normalized = normalized;
    normalized.zeroize();
    return Err(BackupError::password_policy());
  }
  Ok(Some(Zeroizing::new(normalized)))
}

pub async fn create_backup(
  db: &AppDb,
  destination: PathBuf,
  password: Option<String>,
) -> Result<BackupInspection, BackupError> {
  let _write = acquire_write(db).await?;
  let password = normalize_password(password)?;
  paths::validate_backup_destination(db.db_path(), &destination)?;

  let snapshot = sibling_path(db.db_path(), SNAPSHOT_FILE_NAME);
  if snapshot.exists() {
    fs::remove_file(&snapshot).map_err(|_| BackupError::io())?;
  }
  let db_path = db.db_path().to_path_buf();
  let snapshot_for_task = snapshot.clone();
  let vacuumed = tokio::task::spawn_blocking(move || sqlite_raw::vacuum_into(&db_path, &snapshot_for_task))
    .await
    .map_err(|_| BackupError::io())?;
  if let Err(error) = vacuumed {
    let _ = fs::remove_file(&snapshot);
    return Err(error);
  }

  let partial = partial_path(&destination);
  let written = write_snapshot(&snapshot, &partial, password).await;
  if let Err(error) = written {
    let _ = fs::remove_file(&snapshot);
    let _ = fs::remove_file(&partial);
    return Err(error);
  }
  let inspection = written.expect("checked");
  if let Err(error) = paths::replace_file(&partial, &destination) {
    let _ = fs::remove_file(&partial);
    let _ = fs::remove_file(&snapshot);
    return Err(error);
  }
  let _ = fs::remove_file(&snapshot);
  let byte_size = fs::metadata(&destination).map_err(|_| BackupError::io())?.len();
  Ok(BackupInspection {
    byte_size,
    ..inspection
  })
}

pub async fn inspect_backup(
  db: &AppDb,
  source: PathBuf,
  password: Option<String>,
) -> Result<BackupInspection, BackupError> {
  let verify = db.verify_lock();
  let _verify = verify.lock_owned().await;
  let password = normalize_password_for_read(password)?;
  let verify_path = sibling_path(db.db_path(), VERIFY_FILE_NAME);
  let latest = crate::infrastructure::db::latest_schema_version();
  let result = tokio::task::spawn_blocking(move || {
    container::read_container(&source, password.as_ref().map(|value| value.as_str()), &verify_path, latest)
  })
  .await
  .map_err(|_| BackupError::io())?;
  let _ = fs::remove_file(sibling_path(db.db_path(), VERIFY_FILE_NAME));
  result
}

pub async fn restore_backup(
  db: AppDb,
  source: PathBuf,
  password: Option<String>,
) -> Result<BackupInspection, BackupError> {
  restore_backup_inner(db, source, password, None).await
}

#[cfg(test)]
pub async fn restore_backup_at_failpoint(
  db: AppDb,
  source: PathBuf,
  password: Option<String>,
  failpoint: RestoreFailpoint,
) -> Result<BackupInspection, BackupError> {
  restore_backup_inner(db, source, password, Some(failpoint)).await
}

async fn restore_backup_inner(
  db: AppDb,
  source: PathBuf,
  password: Option<String>,
  failpoint: Option<RestoreFailpoint>,
) -> Result<BackupInspection, BackupError> {
  let verify = db.verify_lock();
  let _verify = verify.lock_owned().await;
  let db_path = db.db_path().to_path_buf();
  let password = normalize_password_for_read(password)?;
  let verify_path = sibling_path(&db_path, VERIFY_FILE_NAME);
  let latest = crate::infrastructure::db::latest_schema_version();
  let source_for_read = source.clone();
  let verify_for_read = verify_path.clone();
  let inspected = tokio::task::spawn_blocking(move || {
    container::read_container(
      &source_for_read,
      password.as_ref().map(|value| value.as_str()),
      &verify_for_read,
      latest,
    )
  })
  .await
  .map_err(|_| BackupError::io())?;
  let inspection = match inspected {
    Ok(inspection) => inspection,
    Err(error) => {
      let _ = fs::remove_file(&verify_path);
      return Err(error);
    }
  };

  let mut write = match acquire_write(&db).await {
    Ok(guard) => guard,
    Err(error) => {
      let _ = fs::remove_file(&verify_path);
      return Err(error);
    }
  };
  let incoming = sibling_path(&db_path, INCOMING_FILE_NAME);
  if fs::rename(&verify_path, &incoming).is_err() {
    let _ = fs::remove_file(&verify_path);
    return Err(BackupError::io());
  }
  drop(_verify);

  let has_trigger = tokio::task::spawn_blocking({
    let incoming = incoming.clone();
    move || sqlite_raw::has_trigger(&incoming)
  })
  .await
  .map_err(|_| BackupError::io())??;
  if has_trigger {
    let _ = fs::remove_file(&incoming);
    return Err(BackupError::unsupported_format());
  }

  let incoming_pool = match open_incoming_pool(&incoming).await {
    Ok(pool) => pool,
    Err(error) => {
      let _ = fs::remove_file(&incoming);
      return Err(map_sqlx(&error));
    }
  };
  incoming_pool.close().await;

  write.close().await;
  if failpoint == Some(RestoreFailpoint::AfterCloseBeforeRename) {
    reopen_live(&mut write, &db_path).await?;
    let _ = fs::remove_file(&incoming);
    return Err(BackupError::io());
  }

  paths::remove_sqlite_sidecars(&db_path);
  let pre_restore = sibling_path(&db_path, PRE_RESTORE_FILE_NAME);
  if fs::rename(&db_path, &pre_restore).is_err() {
    reopen_live(&mut write, &db_path).await?;
    let _ = fs::remove_file(&incoming);
    return Err(BackupError::io());
  }
  if fs::rename(&incoming, &db_path).is_err() {
    let _ = fs::rename(&pre_restore, &db_path);
    let _ = fs::remove_file(&incoming);
    reopen_live(&mut write, &db_path).await?;
    return Err(BackupError::io());
  }
  if failpoint == Some(RestoreFailpoint::AfterSwapBeforeOpen) {
    let _ = fs::remove_file(&db_path);
    let _ = fs::rename(&pre_restore, &db_path);
    reopen_live(&mut write, &db_path).await?;
    return Err(BackupError::io());
  }

  match open_pool(&db_path, false).await {
    Ok(pool) => {
      *write = pool;
      let _ = fs::remove_file(&pre_restore);
      Ok(inspection)
    }
    Err(_) => {
      let _ = fs::remove_file(&db_path);
      let _ = fs::rename(&pre_restore, &db_path);
      reopen_live(&mut write, &db_path).await?;
      Err(BackupError::io())
    }
  }
}

async fn write_snapshot(
  snapshot: &Path,
  partial: &Path,
  password: Option<Zeroizing<String>>,
) -> Result<BackupInspection, BackupError> {
  let snapshot_path = snapshot.to_path_buf();
  let partial_path = partial.to_path_buf();
  let inspection = tokio::task::spawn_blocking(move || {
    let encrypted = password.is_some();
    let hash = container::hash_file(&snapshot_path)?;
    let identity = sqlite_raw::read_own_schema(&snapshot_path)?;
    let manifest = Manifest {
      format_version: 1,
      app_version: env!("CARGO_PKG_VERSION").to_string(),
      schema_version: identity.schema_version,
      created_at: Local::now().to_rfc3339(),
      db_file_name: "craft_post.db".to_string(),
      db_sha256: hash,
    };
    container::write_container(
      &snapshot_path,
      &partial_path,
      &manifest,
      password.as_ref().map(|value| value.as_str()),
    )?;
    let byte_size = fs::metadata(&partial_path).map_err(|_| BackupError::io())?.len();
    if byte_size > MAX_CONTAINER_BYTES {
      return Err(BackupError::too_large());
    }
    Ok(BackupInspection {
      created_at: manifest.created_at,
      app_version: manifest.app_version,
      schema_version: manifest.schema_version,
      encrypted,
      byte_size,
    })
  })
  .await
  .map_err(|_| BackupError::io())?;
  inspection
}

async fn acquire_write(db: &AppDb) -> Result<OwnedRwLockWriteGuard<SqlitePool>, BackupError> {
  let lock = db.pool_lock();
  match tokio::time::timeout(WRITE_LOCK_WAIT, lock.write_owned()).await {
    Ok(guard) => Ok(guard),
    Err(_) => Err(BackupError::busy()),
  }
}

async fn reopen_live(
  guard: &mut OwnedRwLockWriteGuard<SqlitePool>,
  db_path: &Path,
) -> Result<(), BackupError> {
  let pool = open_pool(db_path, false).await.map_err(|_| BackupError::reopen_failed())?;
  let closed = std::mem::replace(&mut **guard, pool);
  closed.close().await;
  Ok(())
}

fn normalize_password_for_read(password: Option<String>) -> Result<Option<Zeroizing<String>>, BackupError> {
  let Some(mut raw) = password else {
    return Ok(None);
  };
  let normalized: String = raw.nfc().collect();
  raw.zeroize();
  Ok(Some(Zeroizing::new(normalized)))
}

fn map_sqlx(error: &sqlx::Error) -> BackupError {
  match error {
    sqlx::Error::Io(_) => BackupError::io(),
    _ => BackupError::unsupported_format(),
  }
}
