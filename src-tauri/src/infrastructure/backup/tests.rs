use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Row, SqlitePool};

use crate::infrastructure::db::{
  open_app_db_at, open_pool, sibling_path, AppDb, DB_BUSY_MESSAGE, INCOMING_FILE_NAME,
  PRE_RESTORE_FILE_NAME, VERIFY_FILE_NAME,
};

use super::container::{self, Manifest, MAX_CONTAINER_BYTES};
use super::ops::{self, RestoreFailpoint};
use super::sqlite_raw;

fn temp_dir() -> PathBuf {
  let path = std::env::temp_dir().join(format!("craft-post-backup-{}", uuid::Uuid::new_v4()));
  fs::create_dir_all(&path).unwrap();
  path
}

async fn setup_db(dir: &Path) -> AppDb {
  fs::create_dir_all(dir).unwrap();
  let db_path = dir.join("craft_post.db");
  let pool = open_pool(&db_path, true).await.unwrap();
  AppDb::new(pool, db_path)
}

async fn close_db(db: AppDb) {
  db.pool_lock().write_owned().await.close().await;
}

async fn insert_person(db: &AppDb, last: &str) {
  let pool = db.read().await.unwrap();
  sqlx::query(
    "INSERT INTO address_entries (
       id, primary_last, primary_first, honorific, postal_code,
       prefecture, city, street, created_at, updated_at
     ) VALUES (?, ?, '太郎', '様', '1000001', '東京都', '千代田区', '1-1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
  )
  .bind(uuid::Uuid::new_v4().to_string())
  .bind(last)
  .execute(&*pool)
  .await
  .unwrap();
}

async fn count_named(db: &AppDb, last: &str) -> i64 {
  let pool = db.read().await.unwrap();
  sqlx::query("SELECT COUNT(*) AS n FROM address_entries WHERE primary_last = ?")
    .bind(last)
    .fetch_one(&*pool)
    .await
    .unwrap()
    .get("n")
}

async fn max_version_of(db: &AppDb) -> i64 {
  let pool = db.read().await.unwrap();
  sqlx::query("SELECT MAX(version) AS n FROM _sqlx_migrations WHERE success = 1")
    .fetch_one(&*pool)
    .await
    .unwrap()
    .get("n")
}

fn code_of(result: Result<container::BackupInspection, super::error::BackupError>) -> String {
  result.unwrap_err().code.to_string()
}

fn pack_sqlite(snapshot: &Path, destination: &Path, password: Option<&str>, schema_override: Option<i64>) {
  if let Some(parent) = destination.parent() {
    fs::create_dir_all(parent).unwrap();
  }
  let hash = container::hash_file(snapshot).unwrap();
  let identity = sqlite_raw::read_own_schema(snapshot).unwrap();
  let manifest = Manifest {
    format_version: 1,
    app_version: "0.1.1".to_string(),
    schema_version: schema_override.unwrap_or(identity.schema_version),
    created_at: "2026-10-17T09:00:00+09:00".to_string(),
    db_file_name: "craft_post.db".to_string(),
    db_sha256: hash,
  };
  let partial = destination.with_extension("partial");
  container::write_container(snapshot, &partial, &manifest, password).unwrap();
  fs::rename(partial, destination).unwrap();
}

async fn migrate_upto(path: &Path, version: i64) -> SqlitePool {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).unwrap();
  }
  let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);
  let pool = SqlitePool::connect_with(options).await.unwrap();
  let full = sqlx::migrate!();
  let migrations: Vec<_> = full
    .iter()
    .filter(|migration| migration.version <= version)
    .cloned()
    .collect();
  let migrator = sqlx::migrate::Migrator {
    migrations: std::borrow::Cow::Owned(migrations),
    ignore_missing: false,
    locking: true,
    no_tx: false,
  };
  migrator.run(&pool).await.unwrap();
  pool
}

fn plain_header() -> [u8; 48] {
  let mut header = [0u8; 48];
  header[0..4].copy_from_slice(b"CPBK");
  header[4..6].copy_from_slice(&1u16.to_le_bytes());
  header
}

#[tokio::test]
async fn plain_roundtrip_replaces_rows() {
  let dir = temp_dir();
  let source = setup_db(&dir.join("source")).await;
  insert_person(&source, "往復").await;
  let backup = dir.join("out").join("backup.cpbak");
  fs::create_dir_all(backup.parent().unwrap()).unwrap();
  let created = ops::create_backup(&source, backup.clone(), None).await.unwrap();
  assert!(!created.encrypted);
  assert_eq!(created.schema_version, max_version_of(&source).await);

  let target = setup_db(&dir.join("target")).await;
  insert_person(&target, "現行").await;
  let inspected = ops::inspect_backup(&target, backup.clone(), None).await.unwrap();
  assert_eq!(inspected.schema_version, created.schema_version);
  assert_eq!(inspected.app_version, created.app_version);
  ops::restore_backup(target.clone(), backup.clone(), None).await.unwrap();
  assert_eq!(count_named(&target, "往復").await, 1);
  assert_eq!(count_named(&target, "現行").await, 0);
  assert!(!sibling_path(target.db_path(), VERIFY_FILE_NAME).exists());

  let verify = dir.join("craft_post.db.verify");
  container::read_container(&backup, None, &verify, created.schema_version).unwrap();
  let mut header = [0u8; 16];
  File::open(&verify).unwrap().read_exact(&mut header).unwrap();
  assert_eq!(&header, b"SQLite format 3\0");
  let mut prefix = [0u8; 1];
  File::open(&verify).unwrap().read_exact(&mut prefix).unwrap();
  assert_ne!(prefix[0], b'{');

  close_db(source).await;
  close_db(target).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn encrypted_roundtrip_and_wrong_password_keep_the_target() {
  let dir = temp_dir();
  let source = setup_db(&dir.join("source")).await;
  insert_person(&source, "暗号").await;
  let backup = dir.join("secret.cpbak");
  let password = "password-1".to_string();
  ops::create_backup(&source, backup.clone(), Some(password.clone())).await.unwrap();

  let target = setup_db(&dir.join("target")).await;
  insert_person(&target, "残す").await;
  assert_eq!(
    code_of(ops::inspect_backup(&target, backup.clone(), None).await.map_err(|error| error)),
    "password_required"
  );
  assert!(!sibling_path(target.db_path(), VERIFY_FILE_NAME).exists());
  let wrong = ops::restore_backup(target.clone(), backup.clone(), Some("password-2".to_string()))
    .await
    .unwrap_err();
  assert_eq!(wrong.code, "password_invalid");
  assert_eq!(count_named(&target, "残す").await, 1);

  ops::restore_backup(target.clone(), backup, Some(password)).await.unwrap();
  assert_eq!(count_named(&target, "暗号").await, 1);
  assert_eq!(count_named(&target, "残す").await, 0);
  close_db(source).await;
  close_db(target).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn short_password_does_not_create_a_file() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  let backup = dir.join("short.cpbak");
  let error = ops::create_backup(&db, backup.clone(), Some("1234567".to_string()))
    .await
    .unwrap_err();
  assert_eq!(error.code, "password_policy");
  assert!(!backup.exists());
  close_db(db).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn nfc_equivalent_passwords_roundtrip() {
  let dir = temp_dir();
  let source = setup_db(&dir.join("source")).await;
  insert_person(&source, "正規化").await;
  let backup = dir.join("nfc.cpbak");
  let composed = "\u{304c}".repeat(8);
  let decomposed = "\u{304b}\u{3099}".repeat(8);
  ops::create_backup(&source, backup.clone(), Some(decomposed)).await.unwrap();
  let target = setup_db(&dir.join("target")).await;
  ops::restore_backup(target.clone(), backup, Some(composed)).await.unwrap();
  assert_eq!(count_named(&target, "正規化").await, 1);
  close_db(source).await;
  close_db(target).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn rejects_bad_containers_without_changing_the_database() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  insert_person(&db, "維持").await;
  let latest = max_version_of(&db).await;

  let magic = dir.join("magic.cpbak");
  fs::write(&magic, b"not-a-backup").unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, magic, None).await), "not_backup");

  let flags = dir.join("flags.cpbak");
  let mut header = plain_header();
  header[6] = 2;
  fs::write(&flags, header).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, flags, None).await), "unsupported_format");

  let huge = dir.join("huge.cpbak");
  let file = File::create(&huge).unwrap();
  file.set_len(MAX_CONTAINER_BYTES + 1).unwrap();
  let huge_error = ops::inspect_backup(&db, huge, None).await.unwrap_err();
  assert_eq!(huge_error.code, "unsupported_format");
  assert_eq!(huge_error.message, "バックアップが大きすぎます。");

  let kdf = dir.join("kdf.cpbak");
  let mut header = plain_header();
  header[6] = 1;
  header[8] = 1;
  header[24..28].copy_from_slice(&1u32.to_le_bytes());
  header[28..32].copy_from_slice(&2u32.to_le_bytes());
  header[32..36].copy_from_slice(&1u32.to_le_bytes());
  header[36] = 1;
  fs::write(&kdf, header).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, kdf, Some("password-1".to_string())).await), "unsupported_format");

  let salt = dir.join("salt.cpbak");
  let mut header = plain_header();
  header[6] = 1;
  header[24..28].copy_from_slice(&19_456u32.to_le_bytes());
  header[28..32].copy_from_slice(&2u32.to_le_bytes());
  header[32..36].copy_from_slice(&1u32.to_le_bytes());
  header[36] = 1;
  fs::write(&salt, header).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, salt, Some("password-1".to_string())).await), "unsupported_format");

  let plain_kdf = dir.join("plain-kdf.cpbak");
  let mut header = plain_header();
  header[24..28].copy_from_slice(&1u32.to_le_bytes());
  fs::write(&plain_kdf, header).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, plain_kdf, None).await), "unsupported_format");

  let long_manifest = dir.join("manifest.cpbak");
  let mut bytes = plain_header().to_vec();
  bytes.extend_from_slice(&(MAX_CONTAINER_BYTES as u32).to_le_bytes());
  bytes.extend_from_slice(&70_000u32.to_le_bytes());
  // manifest_len itself is the first u32 after the header. Rewrite those 4 bytes.
  bytes.truncate(48);
  bytes.extend_from_slice(&70_000u32.to_le_bytes());
  fs::write(&long_manifest, bytes).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&db, long_manifest, None).await), "unsupported_format");

  assert!(!sibling_path(db.db_path(), VERIFY_FILE_NAME).exists());
  assert_eq!(count_named(&db, "維持").await, 1);
  let _ = latest;
  close_db(db).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn plain_file_rejects_a_password_and_manifest_mismatch() {
  let dir = temp_dir();
  let source = setup_db(&dir.join("source")).await;
  insert_person(&source, "平文").await;
  let backup = dir.join("plain.cpbak");
  ops::create_backup(&source, backup.clone(), None).await.unwrap();
  let target = setup_db(&dir.join("target")).await;
  insert_person(&target, "維持").await;
  assert_eq!(
    code_of(ops::inspect_backup(&target, backup.clone(), Some("password-1".to_string())).await),
    "password_unexpected"
  );

  let snapshot = dir.join("source").join("craft_post.db");
  let mismatched = dir.join("mismatch.cpbak");
  pack_sqlite(&snapshot, &mismatched, None, Some(1));
  assert_eq!(code_of(ops::inspect_backup(&target, mismatched, None).await), "unsupported_format");
  assert_eq!(count_named(&target, "維持").await, 1);

  let mut bytes = fs::read(&backup).unwrap();
  let last = bytes.len() - 1;
  bytes[last] ^= 0xff;
  let broken = dir.join("broken.cpbak");
  fs::write(&broken, bytes).unwrap();
  assert_eq!(code_of(ops::inspect_backup(&target, broken, None).await), "checksum_mismatch");
  close_db(source).await;
  close_db(target).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn schema_too_new_and_view_and_trigger_do_not_replace_the_database() {
  let dir = temp_dir();
  let live = setup_db(&dir.join("live")).await;
  insert_person(&live, "維持").await;

  let future = dir.join("future.db");
  let pool = open_pool(&future, true).await.unwrap();
  sqlx::query(
    "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
     VALUES (10000, 'future', 1, X'00', 0)",
  )
  .execute(&pool)
  .await
  .unwrap();
  pool.close().await;
  let too_new = dir.join("too-new.cpbak");
  pack_sqlite(&future, &too_new, None, None);
  assert_eq!(code_of(ops::inspect_backup(&live, too_new, None).await), "schema_too_new");

  let view_db = dir.join("view.db");
  let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(&view_db).create_if_missing(true))
    .await
    .unwrap();
  sqlx::query("CREATE VIEW _sqlx_migrations AS SELECT 1 AS version, 1 AS success")
    .execute(&pool)
    .await
    .unwrap();
  pool.close().await;
  let view_backup = dir.join("view.cpbak");
  pack_sqlite(&view_db, &view_backup, None, Some(1));
  assert_eq!(code_of(ops::restore_backup(live.clone(), view_backup, None).await), "unsupported_format");

  let trigger_db = dir.join("trigger.db");
  let pool = open_pool(&trigger_db, true).await.unwrap();
  sqlx::query(
    "CREATE TRIGGER block_insert BEFORE INSERT ON address_entries BEGIN SELECT RAISE(FAIL, 'no'); END",
  )
  .execute(&pool)
  .await
  .unwrap();
  pool.close().await;
  let trigger_backup = dir.join("trigger.cpbak");
  pack_sqlite(&trigger_db, &trigger_backup, None, None);
  assert_eq!(
    code_of(ops::restore_backup(live.clone(), trigger_backup, None).await),
    "unsupported_format"
  );
  assert!(!sibling_path(live.db_path(), INCOMING_FILE_NAME).exists());
  assert_eq!(count_named(&live, "維持").await, 1);
  close_db(live).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn older_backup_is_migrated_and_a_failing_migration_keeps_the_live_database() {
  let dir = temp_dir();
  let old_path = dir.join("old.db");
  let pool = migrate_upto(&old_path, 8).await;
  sqlx::query(
    "INSERT INTO address_entries (
       id, primary_last, primary_first, honorific, postal_code,
       prefecture, city, street, created_at, updated_at
     ) VALUES ('old', '旧版', '太郎', '様', '1000001', '東京都', '千代田区', '1-1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
  )
  .execute(&pool)
  .await
  .unwrap();
  pool.close().await;
  let old_backup = dir.join("old.cpbak");
  pack_sqlite(&old_path, &old_backup, None, None);

  let live = setup_db(&dir.join("live")).await;
  insert_person(&live, "現行").await;
  ops::restore_backup(live.clone(), old_backup, None).await.unwrap();
  assert_eq!(count_named(&live, "旧版").await, 1);
  assert_eq!(count_named(&live, "現行").await, 0);
  assert_eq!(max_version_of(&live).await, crate::infrastructure::db::latest_schema_version());
  let pool = live.read().await.unwrap();
  let columns: i64 = sqlx::query(
    "SELECT COUNT(*) AS n FROM pragma_table_info('address_entries') WHERE name = 'nenga_send_target'",
  )
  .fetch_one(&*pool)
  .await
  .unwrap()
  .get("n");
  drop(pool);
  assert_eq!(columns, 1);

  let broken_path = dir.join("broken.db");
  let pool = open_pool(&broken_path, true).await.unwrap();
  sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 9")
    .execute(&pool)
    .await
    .unwrap();
  sqlx::query(
    "INSERT INTO address_entries (
       id, primary_last, primary_first, honorific, postal_code,
       prefecture, city, street, created_at, updated_at
     ) VALUES ('broken', '失敗', '太郎', '様', '1000001', '東京都', '千代田区', '1-1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
  )
  .execute(&pool)
  .await
  .unwrap();
  pool.close().await;
  let broken_backup = dir.join("broken.cpbak");
  pack_sqlite(&broken_path, &broken_backup, None, None);
  let another = setup_db(&dir.join("another")).await;
  insert_person(&another, "維持").await;
  assert!(ops::restore_backup(another.clone(), broken_backup, None).await.is_err());
  assert_eq!(count_named(&another, "維持").await, 1);
  assert_eq!(count_named(&another, "失敗").await, 0);
  assert!(!sibling_path(another.db_path(), INCOMING_FILE_NAME).exists());
  close_db(live).await;
  close_db(another).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn invalid_paths_and_failed_replace_keep_the_previous_file() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  let db_path = db.db_path().to_path_buf();
  assert_eq!(
    ops::create_backup(&db, db_path.clone(), None).await.unwrap_err().code,
    "invalid_path"
  );
  assert_eq!(
    ops::create_backup(&db, dir.join("backup.txt"), None).await.unwrap_err().code,
    "invalid_path"
  );
  let fresh = dir.join("fresh").join("new.cpbak");
  fs::create_dir_all(fresh.parent().unwrap()).unwrap();
  ops::create_backup(&db, fresh.clone(), None).await.unwrap();
  assert!(fresh.is_file());

  let blocked = dir.join("blocked.cpbak");
  fs::create_dir(&blocked).unwrap();
  fs::write(blocked.join("keep.txt"), b"old").unwrap();
  assert_eq!(ops::create_backup(&db, blocked.clone(), None).await.unwrap_err().code, "io");
  assert!(blocked.join("keep.txt").is_file());
  assert!(!dir.join("blocked.cpbak.partial").exists());
  close_db(db).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn restore_failpoints_reopen_the_original_database() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  insert_person(&db, "残る").await;
  let backup = dir.join("backup.cpbak");
  ops::create_backup(&db, backup.clone(), None).await.unwrap();

  let error = ops::restore_backup_at_failpoint(
    db.clone(),
    backup.clone(),
    None,
    RestoreFailpoint::AfterCloseBeforeRename,
  )
  .await
  .unwrap_err();
  assert_eq!(error.code, "io");
  assert_eq!(count_named(&db, "残る").await, 1);

  let error = ops::restore_backup_at_failpoint(db.clone(), backup, None, RestoreFailpoint::AfterSwapBeforeOpen)
    .await
    .unwrap_err();
  assert_eq!(error.code, "io");
  assert_eq!(count_named(&db, "残る").await, 1);
  close_db(db).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn startup_restores_pre_restore_and_deletes_sidecars() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  insert_person(&db, "復旧").await;
  let db_path = db.db_path().to_path_buf();
  close_db(db).await;
  let pre = sibling_path(&db_path, PRE_RESTORE_FILE_NAME);
  fs::rename(&db_path, &pre).unwrap();
  fs::write(sibling_path(&db_path, VERIFY_FILE_NAME), b"leftover").unwrap();
  fs::write(sibling_path(&db_path, INCOMING_FILE_NAME), b"leftover").unwrap();
  let reopened = open_app_db_at(db_path.clone()).await.unwrap();
  assert_eq!(count_named(&reopened, "復旧").await, 1);
  assert!(!pre.exists());
  assert!(!sibling_path(&db_path, VERIFY_FILE_NAME).exists());
  assert!(!sibling_path(&db_path, INCOMING_FILE_NAME).exists());
  close_db(reopened).await;

  let both = setup_db(&dir.join("both")).await;
  insert_person(&both, "新しい").await;
  let both_path = both.db_path().to_path_buf();
  close_db(both).await;
  fs::copy(&both_path, sibling_path(&both_path, PRE_RESTORE_FILE_NAME)).unwrap();
  let opened = open_app_db_at(both_path.clone()).await.unwrap();
  assert_eq!(count_named(&opened, "新しい").await, 1);
  assert!(!sibling_path(&both_path, PRE_RESTORE_FILE_NAME).exists());
  close_db(opened).await;
  let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn read_lock_times_out_while_a_backup_holds_the_write_lock() {
  let dir = temp_dir();
  let db = setup_db(&dir).await;
  let _write = db.pool_lock().write_owned().await;
  let started = Instant::now();
  let error = match db.read().await {
    Ok(_) => panic!("read lock should time out"),
    Err(error) => error,
  };
  assert_eq!(error, DB_BUSY_MESSAGE);
  assert!(started.elapsed() >= Duration::from_secs(3));
  assert!(started.elapsed() < Duration::from_secs(6));
  drop(_write);
  close_db(db).await;
  let _ = fs::remove_dir_all(dir);
}

#[test]
fn container_over_the_limit_is_rejected() {
  let dir = temp_dir();
  let path = dir.join("big.partial");
  OpenOptions::new().create(true).write(true).open(&path).unwrap().set_len(MAX_CONTAINER_BYTES + 1).unwrap();
  let error = container::read_container(&path, None, &dir.join("verify"), 9).unwrap_err();
  assert_eq!(error.message, "バックアップが大きすぎます。");
  assert!(!dir.join("verify").exists());
  assert!(path.is_file());
  let _ = fs::remove_dir_all(dir);
}
