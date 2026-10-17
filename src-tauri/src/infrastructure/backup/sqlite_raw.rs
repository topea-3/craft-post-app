use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

use libsqlite3_sys::{
  sqlite3, sqlite3_busy_timeout, sqlite3_close, sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type,
  sqlite3_db_config, sqlite3_extended_result_codes, sqlite3_finalize, sqlite3_open_v2, sqlite3_prepare_v2,
  sqlite3_step, SQLITE_BUSY, SQLITE_DBCONFIG_DEFENSIVE, SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION,
  SQLITE_DONE, SQLITE_NULL, SQLITE_OK, SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE, SQLITE_OPEN_URI,
  SQLITE_ROW,
};

use super::error::BackupError;

const SQLITE_HEADER: &[u8] = b"SQLite format 3\0";

pub fn sqlite_header() -> &'static [u8] {
  SQLITE_HEADER
}

pub fn vacuum_into(source: &Path, destination: &Path) -> Result<(), BackupError> {
  let source = path_cstring(source)?;
  let mut db = ptr::null_mut();
  let status = unsafe {
    sqlite3_open_v2(
      source.as_ptr(),
      &mut db,
      SQLITE_OPEN_READWRITE,
      ptr::null(),
    )
  };
  if db.is_null() {
    return Err(BackupError::io());
  }
  let connection = RawDb { db };
  if status != SQLITE_OK {
    return Err(BackupError::io());
  }
  unsafe {
    sqlite3_extended_result_codes(connection.db, 1);
    sqlite3_busy_timeout(connection.db, 5_000);
  }
  let destination = destination.to_str().ok_or_else(BackupError::io)?;
  let sql = format!("VACUUM INTO '{}'", destination.replace('\'', "''"));
  let sql = CString::new(sql).map_err(|_| BackupError::io())?;
  let exec_status = unsafe { sqlite3_exec_status(connection.db, sql.as_ptr()) };
  if is_busy(exec_status) {
    return Err(BackupError::busy());
  }
  if exec_status != SQLITE_OK {
    return Err(BackupError::io());
  }
  Ok(())
}

pub struct MigrationIdentity {
  pub schema_version: i64,
}

/// 外部ファイル向け。immutable、拡張無効、DEFENSIVE、trusted_schema=OFF。
/// 許可する問い合わせは `_sqlx_migrations` の型と最大 version だけ。
pub fn read_schema_identity(path: &Path) -> Result<MigrationIdentity, BackupError> {
  let connection = open_restricted(path)?;
  let type_sql = CString::new("SELECT type FROM sqlite_master WHERE name = '_sqlx_migrations'")
    .map_err(|_| BackupError::io())?;
  let type_name = query_text(connection.db, type_sql.as_ptr())?;
  if type_name.as_deref() != Some("table") {
    return Err(BackupError::unsupported_format());
  }
  let version_sql =
    CString::new("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1").map_err(|_| BackupError::io())?;
  let version = query_i64(connection.db, version_sql.as_ptr())?;
  let Some(version) = version else {
    return Err(BackupError::unsupported_format());
  };
  Ok(MigrationIdentity {
    schema_version: version,
  })
}

/// アプリ自身が VACUUM INTO したスナップショット。外部ファイル用の制限は掛けない。
pub fn read_own_schema(path: &Path) -> Result<MigrationIdentity, BackupError> {
  let text = path_cstring(path)?;
  let mut db = ptr::null_mut();
  let status = unsafe { sqlite3_open_v2(text.as_ptr(), &mut db, SQLITE_OPEN_READONLY, ptr::null()) };
  if db.is_null() || status != SQLITE_OK {
    if !db.is_null() {
      unsafe { sqlite3_close(db) };
    }
    return Err(BackupError::io());
  }
  let connection = RawDb { db };
  let version_sql =
    CString::new("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1").map_err(|_| BackupError::io())?;
  let version = query_i64(connection.db, version_sql.as_ptr())?.ok_or_else(BackupError::io)?;
  Ok(MigrationIdentity {
    schema_version: version,
  })
}

pub fn has_trigger(path: &Path) -> Result<bool, BackupError> {
  let connection = open_restricted(path)?;
  let sql = CString::new("SELECT 1 FROM sqlite_master WHERE type = 'trigger' LIMIT 1").map_err(|_| BackupError::io())?;
  Ok(query_i64(connection.db, sql.as_ptr())?.is_some())
}

struct RawDb {
  db: *mut sqlite3,
}

impl Drop for RawDb {
  fn drop(&mut self) {
    unsafe {
      sqlite3_close(self.db);
    }
  }
}

fn open_restricted(path: &Path) -> Result<RawDb, BackupError> {
  let uri = immutable_uri(path)?;
  let uri = CString::new(uri).map_err(|_| BackupError::io())?;
  let mut db = ptr::null_mut();
  let status = unsafe {
    sqlite3_open_v2(
      uri.as_ptr(),
      &mut db,
      SQLITE_OPEN_READONLY | SQLITE_OPEN_URI,
      ptr::null(),
    )
  };
  if db.is_null() || status != SQLITE_OK {
    if !db.is_null() {
      unsafe { sqlite3_close(db) };
    }
    return Err(BackupError::unsupported_format());
  }
  let connection = RawDb { db };
  let defensive = unsafe {
    sqlite3_db_config(
      connection.db,
      SQLITE_DBCONFIG_DEFENSIVE,
      1,
      ptr::null_mut::<c_int>(),
    )
  };
  let extensions = unsafe {
    sqlite3_db_config(
      connection.db,
      SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION,
      0,
      ptr::null_mut::<c_int>(),
    )
  };
  if defensive != SQLITE_OK || extensions != SQLITE_OK {
    return Err(BackupError::unsupported_format());
  }
  let pragma = CString::new("PRAGMA trusted_schema=OFF").map_err(|_| BackupError::io())?;
  let pragma_status = unsafe { sqlite3_exec_status(connection.db, pragma.as_ptr()) };
  if pragma_status != SQLITE_OK {
    return Err(BackupError::unsupported_format());
  }
  Ok(connection)
}

fn immutable_uri(path: &Path) -> Result<String, BackupError> {
  let text = path.to_str().ok_or_else(BackupError::io)?.replace('\\', "/");
  let mut encoded = String::new();
  for byte in text.bytes() {
    match byte {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => {
        encoded.push(byte as char);
      }
      _ => encoded.push_str(&format!("%{byte:02X}")),
    }
  }
  Ok(format!("file:///{encoded}?immutable=1"))
}

fn path_cstring(path: &Path) -> Result<CString, BackupError> {
  let text = path.to_str().ok_or_else(BackupError::io)?;
  CString::new(text).map_err(|_| BackupError::io())
}

fn query_text(db: *mut sqlite3, sql: *const c_char) -> Result<Option<String>, BackupError> {
  let mut statement = ptr::null_mut();
  let prepare_status = unsafe { sqlite3_prepare_v2(db, sql, -1, &mut statement, ptr::null_mut()) };
  if prepare_status != SQLITE_OK {
    return Err(BackupError::unsupported_format());
  }
  let step = unsafe { sqlite3_step(statement) };
  let value = if step == SQLITE_ROW {
    let text = unsafe { sqlite3_column_text(statement, 0) };
    if text.is_null() {
      None
    } else {
      let cstr = unsafe { std::ffi::CStr::from_ptr(text as *const c_char) };
      Some(cstr.to_string_lossy().into_owned())
    }
  } else if step == SQLITE_DONE {
    None
  } else {
    unsafe { sqlite3_finalize(statement) };
    return Err(BackupError::unsupported_format());
  };
  unsafe { sqlite3_finalize(statement) };
  Ok(value)
}

fn query_i64(db: *mut sqlite3, sql: *const c_char) -> Result<Option<i64>, BackupError> {
  let mut statement = ptr::null_mut();
  let prepare_status = unsafe { sqlite3_prepare_v2(db, sql, -1, &mut statement, ptr::null_mut()) };
  if prepare_status != SQLITE_OK {
    return Err(BackupError::unsupported_format());
  }
  let step = unsafe { sqlite3_step(statement) };
  let value = if step == SQLITE_ROW {
    let column_type = unsafe { sqlite3_column_type(statement, 0) };
    if column_type == SQLITE_NULL {
      None
    } else {
      Some(unsafe { sqlite3_column_int64(statement, 0) })
    }
  } else if step == SQLITE_DONE {
    None
  } else {
    unsafe { sqlite3_finalize(statement) };
    return Err(BackupError::unsupported_format());
  };
  unsafe { sqlite3_finalize(statement) };
  Ok(value)
}

unsafe fn sqlite3_exec_status(db: *mut sqlite3, sql: *const c_char) -> c_int {
  unsafe extern "C" {
    fn sqlite3_exec(
      db: *mut sqlite3,
      sql: *const c_char,
      callback: *mut std::ffi::c_void,
      arg: *mut std::ffi::c_void,
      errmsg: *mut *mut c_char,
    ) -> c_int;
  }
  unsafe {
    sqlite3_exec(db, sql, ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
  }
}

fn is_busy(status: c_int) -> bool {
  status == SQLITE_BUSY || status & 0xff == SQLITE_BUSY
}
