use std::fs;
use std::fs::OpenOptions;
#[cfg(not(windows))]
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::infrastructure::db::{
  sibling_path, DB_FILE_NAME, INCOMING_FILE_NAME, PRE_RESTORE_FILE_NAME, SNAPSHOT_FILE_NAME,
  VERIFY_FILE_NAME,
};

use super::error::BackupError;

pub fn validate_backup_destination(db_path: &Path, destination: &Path) -> Result<(), BackupError> {
  let file_name = destination.file_name().ok_or_else(BackupError::invalid_path)?;
  if !is_cpbak(file_name) {
    return Err(BackupError::invalid_path());
  }
  let parent = destination
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
    .ok_or_else(BackupError::io)?;
  if !parent.is_dir() {
    return Err(BackupError::io());
  }
  let canonical_parent = parent.canonicalize().map_err(|_| BackupError::io())?;
  let resolved = if destination.exists() {
    destination.canonicalize().map_err(|_| BackupError::io())?
  } else {
    canonical_parent.join(file_name)
  };

  let app_data = db_path.parent().ok_or_else(BackupError::io)?;
  for candidate in forbidden_paths(db_path, app_data)? {
    if paths_equal(&resolved, &candidate) {
      return Err(BackupError::invalid_path());
    }
  }
  Ok(())
}

pub fn partial_path(destination: &Path) -> PathBuf {
  let mut name = destination
    .file_name()
    .unwrap_or_default()
    .to_os_string();
  name.push(".partial");
  destination.with_file_name(name)
}

pub fn fsync_file(path: &Path) -> Result<(), BackupError> {
  let file = OpenOptions::new().write(true).open(path).map_err(|_| BackupError::io())?;
  file.sync_all().map_err(|_| BackupError::io())
}

pub fn replace_file(partial: &Path, destination: &Path) -> Result<(), BackupError> {
  fsync_file(partial)?;
  #[cfg(windows)]
  {
    move_file_replace(partial, destination)?;
  }
  #[cfg(not(windows))]
  {
    fs::rename(partial, destination).map_err(|_| BackupError::io())?;
    if let Some(parent) = destination.parent() {
      if let Err(error) = sync_parent(parent) {
        log::warn!("replaced backup file but directory sync failed: {error}");
      }
    }
  }
  Ok(())
}

pub fn remove_sqlite_sidecars(db_path: &Path) {
  for suffix in ["-journal", "-wal", "-shm"] {
    let mut name = db_path
      .file_name()
      .unwrap_or_default()
      .to_os_string();
    name.push(suffix);
    if let Some(parent) = db_path.parent() {
      let path = parent.join(name);
      let _ = fs::remove_file(path);
    }
  }
}

fn forbidden_paths(db_path: &Path, app_data: &Path) -> Result<Vec<PathBuf>, BackupError> {
  let mut paths = Vec::new();
  paths.push(comparable_path(db_path, app_data, DB_FILE_NAME)?);
  for name in [
    INCOMING_FILE_NAME,
    PRE_RESTORE_FILE_NAME,
    SNAPSHOT_FILE_NAME,
    VERIFY_FILE_NAME,
  ] {
    paths.push(comparable_path(&sibling_path(db_path, name), app_data, name)?);
  }
  Ok(paths)
}

fn comparable_path(path: &Path, app_data: &Path, name: &str) -> Result<PathBuf, BackupError> {
  if path.exists() {
    path.canonicalize().map_err(|_| BackupError::io())
  } else {
    let parent = app_data.canonicalize().map_err(|_| BackupError::io())?;
    Ok(parent.join(name))
  }
}

fn is_cpbak(name: &std::ffi::OsStr) -> bool {
  let text = name.to_string_lossy();
  text.len() > ".cpbak".len() && text[text.len() - ".cpbak".len()..].eq_ignore_ascii_case(".cpbak")
}

fn paths_equal(left: &Path, right: &Path) -> bool {
  #[cfg(windows)]
  {
    left
      .to_string_lossy()
      .eq_ignore_ascii_case(&right.to_string_lossy())
  }
  #[cfg(not(windows))]
  {
    left == right
  }
}

#[cfg(not(windows))]
fn sync_parent(parent: &Path) -> Result<(), std::io::Error> {
  let dir = File::open(parent)?;
  dir.sync_all()
}

#[cfg(windows)]
fn move_file_replace(from: &Path, to: &Path) -> Result<(), BackupError> {
  use std::os::windows::ffi::OsStrExt;

  const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
  unsafe extern "system" {
    fn MoveFileExW(existing: *const u16, new_name: *const u16, flags: u32) -> i32;
  }

  let from_wide: Vec<u16> = from.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
  let to_wide: Vec<u16> = to.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
  let moved = unsafe { MoveFileExW(from_wide.as_ptr(), to_wide.as_ptr(), MOVEFILE_REPLACE_EXISTING) };
  if moved == 0 {
    Err(BackupError::io())
  } else {
    Ok(())
  }
}
