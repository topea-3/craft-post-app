#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;
  use uuid::Uuid;

  use crate::domain::print::postcard_send::PostcardSend;
  use crate::domain::print::postcard_send_repository::{
    PostcardSendRepository, PostcardSendRepositoryError,
  };
  use crate::domain::print::postcard_type::PostcardType;
  use crate::infrastructure::print::sqlx_postcard_send_repository::SqlxPostcardSendRepository;

  async fn setup_pool() -> SqlitePool {
    use std::path::PathBuf;
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let migrations_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations_path)
      .await
      .expect("migrations dir");
    migrator.run(&pool).await.unwrap();
    pool
  }

  async fn seed_address(pool: &SqlitePool, id: Uuid) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id, primary_last, primary_first, primary_kana_last, primary_kana_first,
          honorific, postal_code, prefecture, city, street, building, memo, archived_at, created_at, updated_at
        )
        VALUES (?, '佐藤', '一郎', NULL, NULL, '様', '1234567', '東京都', '千代田区', '1-1-1', NULL, NULL, NULL, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
  }

  async fn seed_sender(pool: &SqlitePool, id: Uuid) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO sender_entries (
          id, label, primary_last, primary_first, primary_kana_last, primary_kana_first,
          postal_code, prefecture, city, street, building, phone_number, archived_at, created_at, updated_at
        )
        VALUES (?, '自宅', '山田', '太郎', NULL, NULL, '1234567', '東京都', '渋谷区', '神南 1-1-1', NULL, NULL, NULL, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
  }

  fn sample_send(
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
  ) -> PostcardSend {
    PostcardSend::create_new(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      r#"{"sender_entry_id":"x"}"#.to_string(),
      r#"{"address_entry_id":"y"}"#.to_string(),
      PostcardType::Nenga,
    )
  }

  #[tokio::test]
  async fn create_batch_success() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id).await;
    seed_sender(&pool, sender_id).await;

    let print_job_id = Uuid::new_v4();
    let repo = SqlxPostcardSendRepository::new(pool.clone());
    repo
      .create_batch(&[sample_send(print_job_id, address_id, sender_id)])
      .await
      .expect("create_batch");

    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM postcard_sends WHERE print_job_id = ? AND deleted_at IS NULL",
    )
    .bind(print_job_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
  }

  #[tokio::test]
  async fn create_batch_duplicate_returns_conflict() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id).await;
    seed_sender(&pool, sender_id).await;

    let print_job_id = Uuid::new_v4();
    let repo = SqlxPostcardSendRepository::new(pool);
    let first = sample_send(print_job_id, address_id, sender_id);
    let second = sample_send(print_job_id, address_id, sender_id);

    repo.create_batch(&[first]).await.expect("first insert");
    let err = repo
      .create_batch(&[second])
      .await
      .expect_err("duplicate should conflict");
    assert!(matches!(err, PostcardSendRepositoryError::Conflict));
  }
}
