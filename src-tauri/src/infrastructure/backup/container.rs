use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use super::error::BackupError;
use super::gcm::{derive_key, tags_equal, GcmSession};
use super::sqlite_raw::{self, sqlite_header};

pub const MAX_CONTAINER_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u32 = 64 * 1024;
const HEADER_LEN: usize = 48;
const DB_FILE_NAME: &str = "craft_post.db";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
  pub format_version: u32,
  pub app_version: String,
  pub schema_version: i64,
  pub created_at: String,
  pub db_file_name: String,
  pub db_sha256: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInspection {
  pub created_at: String,
  pub app_version: String,
  pub schema_version: i64,
  pub encrypted: bool,
  pub byte_size: u64,
}

struct Header {
  format_version: u16,
  encrypted: bool,
  salt: [u8; 16],
  nonce: [u8; 12],
  raw: [u8; HEADER_LEN],
}

pub fn hash_file(path: &Path) -> Result<String, BackupError> {
  let mut file = File::open(path).map_err(|_| BackupError::io())?;
  let mut hasher = Sha256::new();
  let mut buffer = [0u8; 64 * 1024];
  loop {
    let read = file.read(&mut buffer).map_err(|_| BackupError::io())?;
    if read == 0 {
      break;
    }
    hasher.update(&buffer[..read]);
  }
  Ok(hex::encode(hasher.finalize()))
}

pub fn write_container(
  snapshot: &Path,
  partial: &Path,
  manifest: &Manifest,
  password: Option<&str>,
) -> Result<(), BackupError> {
  let manifest_bytes = serde_json::to_vec(manifest).map_err(|_| BackupError::io())?;
  if manifest_bytes.len() > MAX_MANIFEST_BYTES as usize {
    return Err(BackupError::unsupported_format());
  }
  let header = Header::for_write(password.is_some())?;
  let mut output = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(partial)
    .map_err(|_| BackupError::io())?;
  output.write_all(&header.raw).map_err(|_| BackupError::io())?;

  let mut prefix = Vec::with_capacity(4 + manifest_bytes.len());
  prefix.extend_from_slice(&(manifest_bytes.len() as u32).to_le_bytes());
  prefix.extend_from_slice(&manifest_bytes);

  if let Some(password) = password {
    let mut key = derive_key(password, &header.salt)?;
    let mut session = GcmSession::new(&key, &header.nonce, &header.raw);
    key.zeroize();
    session.encrypt(&mut prefix);
    output.write_all(&prefix).map_err(|_| BackupError::io())?;
    prefix.zeroize();

    let mut input = File::open(snapshot).map_err(|_| BackupError::io())?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
      let read = input.read(&mut buffer).map_err(|_| BackupError::io())?;
      if read == 0 {
        break;
      }
      session.encrypt(&mut buffer[..read]);
      output.write_all(&buffer[..read]).map_err(|_| BackupError::io())?;
    }
    let tag = session.finish();
    output.write_all(&tag).map_err(|_| BackupError::io())?;
  } else {
    output.write_all(&prefix).map_err(|_| BackupError::io())?;
    let mut input = File::open(snapshot).map_err(|_| BackupError::io())?;
    std::io::copy(&mut input, &mut output).map_err(|_| BackupError::io())?;
  }
  output.flush().map_err(|_| BackupError::io())?;
  Ok(())
}

pub fn read_container(
  source: &Path,
  password: Option<&str>,
  verify_path: &Path,
  latest_schema: i64,
) -> Result<BackupInspection, BackupError> {
  let mut cleanup = VerifyCleanup {
    path: verify_path,
    keep: false,
  };
  let length = fs::metadata(source).map_err(|_| BackupError::io())?.len();
  if length > MAX_CONTAINER_BYTES {
    return Err(BackupError::too_large());
  }
  if length < HEADER_LEN as u64 {
    return Err(BackupError::not_backup());
  }

  let mut file = File::open(source).map_err(|_| BackupError::io())?;
  let mut header_bytes = [0u8; HEADER_LEN];
  file.read_exact(&mut header_bytes).map_err(|_| BackupError::io())?;
  let header = Header::parse(&header_bytes)?;

  if header.encrypted && password.is_none() {
    return Err(BackupError::password_required());
  }
  if !header.encrypted && password.is_some() {
    return Err(BackupError::password_unexpected());
  }

  let plain_len = if header.encrypted {
    length
      .checked_sub(HEADER_LEN as u64 + 16)
      .ok_or_else(BackupError::unsupported_format)?
  } else {
    length - HEADER_LEN as u64
  };

  let manifest = if header.encrypted {
    let password = password.ok_or_else(BackupError::password_required)?;
    let mut key = derive_key(password, &header.salt)?;
    let mut session = GcmSession::new(&key, &header.nonce, &header.raw);
    key.zeroize();
    decrypt_body(&mut file, &mut session, plain_len, verify_path)?
  } else {
    read_plain_body(&mut file, plain_len, verify_path)?
  };

  if manifest.format_version != u32::from(header.format_version) {
    return Err(BackupError::unsupported_format());
  }
  super::paths::fsync_file(verify_path)?;
  if !file_starts_with(verify_path, sqlite_header())? {
    return Err(BackupError::checksum_mismatch());
  }
  let actual_hash = hash_file(verify_path)?;
  if actual_hash != manifest.db_sha256 {
    return Err(BackupError::checksum_mismatch());
  }

  let identity = sqlite_raw::read_schema_identity(verify_path)?;
  if identity.schema_version != manifest.schema_version {
    return Err(BackupError::unsupported_format());
  }
  if sqlite_raw::has_trigger(verify_path)? {
    return Err(BackupError::unsupported_format());
  }
  if identity.schema_version > latest_schema {
    return Err(BackupError::schema_too_new());
  }

  cleanup.keep = true;
  Ok(BackupInspection {
    created_at: manifest.created_at,
    app_version: manifest.app_version,
    schema_version: manifest.schema_version,
    encrypted: header.encrypted,
    byte_size: length,
  })
}

fn decrypt_exact(file: &mut File, session: &mut GcmSession, count: usize) -> Result<Vec<u8>, BackupError> {
  let mut buffer = vec![0u8; count];
  file.read_exact(&mut buffer).map_err(|_| BackupError::io())?;
  session.decrypt(&mut buffer);
  Ok(buffer)
}

fn decrypt_body(
  file: &mut File,
  session: &mut GcmSession,
  mut cipher_len: u64,
  verify_path: &Path,
) -> Result<Manifest, BackupError> {
  if cipher_len < 4 {
    return Err(BackupError::unsupported_format());
  }
  let length_bytes = decrypt_exact(file, session, 4)?;
  cipher_len -= 4;
  let manifest_len = u32::from_le_bytes(length_bytes.as_slice().try_into().expect("4 bytes"));
  let length_ok = manifest_len <= MAX_MANIFEST_BYTES && u64::from(manifest_len) <= cipher_len;
  let manifest_bytes = if length_ok {
    let bytes = decrypt_exact(file, session, manifest_len as usize)?;
    cipher_len -= u64::from(manifest_len);
    Some(bytes)
  } else {
    None
  };

  let mut output = if manifest_bytes.is_some() {
    Some(
      OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(verify_path)
        .map_err(|_| BackupError::io())?,
    )
  } else {
    None
  };
  let mut buffer = [0u8; 64 * 1024];
  while cipher_len > 0 {
    let chunk = (buffer.len() as u64).min(cipher_len) as usize;
    file.read_exact(&mut buffer[..chunk]).map_err(|_| BackupError::io())?;
    session.decrypt(&mut buffer[..chunk]);
    if let Some(output) = output.as_mut() {
      output.write_all(&buffer[..chunk]).map_err(|_| BackupError::io())?;
    }
    cipher_len -= chunk as u64;
  }
  if let Some(output) = output.as_mut() {
    output.flush().map_err(|_| BackupError::io())?;
  }
  drop(output);

  let mut tag = [0u8; 16];
  file.read_exact(&mut tag).map_err(|_| BackupError::io())?;
  let actual = std::mem::replace(session, GcmSession::new(&[0u8; 32], &[0u8; 12], &[])).finish();
  if !tags_equal(&tag, &actual) {
    return Err(BackupError::password_invalid());
  }
  let manifest_bytes = manifest_bytes.ok_or_else(BackupError::unsupported_format)?;
  parse_manifest(&manifest_bytes)
}

fn read_plain_body(file: &mut File, mut plain_len: u64, verify_path: &Path) -> Result<Manifest, BackupError> {
  if plain_len < 4 {
    return Err(BackupError::unsupported_format());
  }
  let mut length_bytes = [0u8; 4];
  file.read_exact(&mut length_bytes).map_err(|_| BackupError::io())?;
  plain_len -= 4;
  let manifest_len = u32::from_le_bytes(length_bytes);
  if manifest_len > MAX_MANIFEST_BYTES || u64::from(manifest_len) > plain_len {
    return Err(BackupError::unsupported_format());
  }
  let mut manifest_bytes = vec![0u8; manifest_len as usize];
  file.read_exact(&mut manifest_bytes).map_err(|_| BackupError::io())?;
  plain_len -= u64::from(manifest_len);
  let manifest = parse_manifest(&manifest_bytes)?;

  let mut output = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(verify_path)
    .map_err(|_| BackupError::io())?;
  let mut remaining = plain_len;
  let mut buffer = [0u8; 64 * 1024];
  while remaining > 0 {
    let chunk = (buffer.len() as u64).min(remaining) as usize;
    file.read_exact(&mut buffer[..chunk]).map_err(|_| BackupError::io())?;
    output.write_all(&buffer[..chunk]).map_err(|_| BackupError::io())?;
    remaining -= chunk as u64;
  }
  output.flush().map_err(|_| BackupError::io())?;
  Ok(manifest)
}

fn parse_manifest(bytes: &[u8]) -> Result<Manifest, BackupError> {
  let manifest: Manifest = serde_json::from_slice(bytes).map_err(|_| BackupError::unsupported_format())?;
  if manifest.format_version != 1
    || manifest.app_version.is_empty()
    || manifest.app_version.chars().count() > 32
    || manifest.schema_version < 1
    || manifest.created_at.is_empty()
    || manifest.db_file_name != DB_FILE_NAME
    || !is_sha256_hex(&manifest.db_sha256)
  {
    return Err(BackupError::unsupported_format());
  }
  Ok(manifest)
}

fn is_sha256_hex(value: &str) -> bool {
  value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn file_starts_with(path: &Path, prefix: &[u8]) -> Result<bool, BackupError> {
  let mut file = File::open(path).map_err(|_| BackupError::io())?;
  let mut buffer = vec![0u8; prefix.len()];
  match file.read_exact(&mut buffer) {
    Ok(()) => Ok(buffer == prefix),
    Err(_) => Ok(false),
  }
}

struct VerifyCleanup<'a> {
  path: &'a Path,
  keep: bool,
}

impl Drop for VerifyCleanup<'_> {
  fn drop(&mut self) {
    if !self.keep {
      let _ = fs::remove_file(self.path);
    }
  }
}

impl Header {
  fn for_write(encrypted: bool) -> Result<Self, BackupError> {
    let mut raw = [0u8; HEADER_LEN];
    raw[0..4].copy_from_slice(b"CPBK");
    raw[4..6].copy_from_slice(&1u16.to_le_bytes());
    let (salt, nonce, memory, iterations, parallelism, flags): ([u8; 16], [u8; 12], u32, u32, u32, u16) =
      if encrypted {
        (random_nonzero()?, random_nonzero()?, 19_456, 2, 1, 1)
      } else {
        ([0u8; 16], [0u8; 12], 0, 0, 0, 0)
      };
    raw[6..8].copy_from_slice(&flags.to_le_bytes());
    raw[8..24].copy_from_slice(&salt);
    raw[24..28].copy_from_slice(&memory.to_le_bytes());
    raw[28..32].copy_from_slice(&iterations.to_le_bytes());
    raw[32..36].copy_from_slice(&parallelism.to_le_bytes());
    raw[36..48].copy_from_slice(&nonce);
    Ok(Self {
      format_version: 1,
      encrypted,
      salt,
      nonce,
      raw,
    })
  }

  fn parse(raw: &[u8; HEADER_LEN]) -> Result<Self, BackupError> {
    if &raw[0..4] != b"CPBK" {
      return Err(BackupError::not_backup());
    }
    let format_version = u16::from_le_bytes(raw[4..6].try_into().expect("2 bytes"));
    let flags = u16::from_le_bytes(raw[6..8].try_into().expect("2 bytes"));
    if format_version != 1 || flags & !1 != 0 {
      return Err(BackupError::unsupported_format());
    }
    let encrypted = flags & 1 == 1;
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&raw[8..24]);
    let memory_kib = u32::from_le_bytes(raw[24..28].try_into().expect("4 bytes"));
    let iterations = u32::from_le_bytes(raw[28..32].try_into().expect("4 bytes"));
    let parallelism = u32::from_le_bytes(raw[32..36].try_into().expect("4 bytes"));
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&raw[36..48]);
    let salt_zero = salt.iter().all(|byte| *byte == 0);
    let nonce_zero = nonce.iter().all(|byte| *byte == 0);
    if encrypted {
      if memory_kib != 19_456 || iterations != 2 || parallelism != 1 || salt_zero || nonce_zero {
        return Err(BackupError::unsupported_format());
      }
    } else if !salt_zero || !nonce_zero || memory_kib != 0 || iterations != 0 || parallelism != 0 {
      return Err(BackupError::unsupported_format());
    }
    Ok(Self {
      format_version,
      encrypted,
      salt,
      nonce,
      raw: *raw,
    })
  }
}

fn random_nonzero<const N: usize>() -> Result<[u8; N], BackupError> {
  for _ in 0..3 {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).map_err(|_| BackupError::io())?;
    if bytes.iter().any(|byte| *byte != 0) {
      return Ok(bytes);
    }
  }
  Err(BackupError::io())
}
