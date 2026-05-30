#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;
  use sqlx::Row;
  use uuid::Uuid;

  use crate::domain::sender::sender_entry_repository::SenderEntryRepository;
  use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

  use crate::{
    create_sender_entry_impl, set_sender_for_address_entry_impl, update_sender_entry_impl,
    update_sender_entry_links_impl, AddressDto, PersonNameDto, SenderEntryDtoInput,
  };

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

  async fn insert_address_entry(pool: &SqlitePool, id: Uuid, archived: bool) {
    let now = chrono::Utc::now().to_rfc3339();
    let archived_at: Option<&str> = if archived { Some(now.as_str()) } else { None };
    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id, primary_last, primary_first, primary_kana_last, primary_kana_first,
          honorific, postal_code, prefecture, city, street, building, memo, archived_at, created_at, updated_at
        )
        VALUES (?, '佐藤', '一郎', NULL, NULL, '様', '1234567', '東京都', '千代田区', '1-1-1', NULL, NULL, ?, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind(archived_at)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
  }

  async fn fetch_sender_id_by_label(pool: &SqlitePool, label: &str) -> String {
    let row = sqlx::query(
      r#"
        SELECT id
        FROM sender_entries
        WHERE archived_at IS NULL AND label = ?
        LIMIT 1
      "#,
    )
    .bind(label)
    .fetch_one(pool)
    .await
    .unwrap();
    row.get::<String, _>("id")
  }

  fn sample_sender_dto(label: &str) -> SenderEntryDtoInput {
    SenderEntryDtoInput {
      label: label.to_string(),
      primary_name: PersonNameDto {
        last: "山田".to_string(),
        first: "太郎".to_string(),
        kana_last: None,
        kana_first: None,
      },
      co_recipients: vec![],
      postal_code: "1234567".to_string(),
      address: AddressDto {
        prefecture: "東京都".to_string(),
        city: "渋谷区".to_string(),
        street: "神南 1-1-1".to_string(),
        building: None,
      },
      phone_number: None,
    }
  }

  #[tokio::test]
  async fn create_sender_entry_duplicate_label_is_validation_error() {
    let pool = setup_pool().await;

    create_sender_entry_impl(&pool, sample_sender_dto("自宅"))
      .await
      .expect("first create should succeed");
    let err = create_sender_entry_impl(&pool, sample_sender_dto("自宅"))
      .await
      .expect_err("duplicate label should fail");

    assert!(err.contains("このラベルは既に使用されています"));
  }

  #[tokio::test]
  async fn create_sender_entry_detects_existing_active_label_in_db() {
    let pool = setup_pool().await;
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
      r#"
        INSERT INTO sender_entries (
          id, label, primary_last, primary_first, primary_kana_last, primary_kana_first,
          postal_code, prefecture, city, street, building, phone_number, archived_at, created_at, updated_at
        )
        VALUES (?, '自宅', '山田', '太郎', NULL, NULL, '1234567', '東京都', '渋谷区', '神南 1-1-1', NULL, NULL, NULL, ?, ?)
      "#,
    )
    .bind(&id)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("seed active sender");

    let err = create_sender_entry_impl(&pool, sample_sender_dto("自宅"))
      .await
      .expect_err("duplicate active label should fail");

    assert!(err.contains("このラベルは既に使用されています"));
  }

  #[tokio::test]
  async fn sender_repository_duplicate_active_label_maps_to_duplicate_error() {
    use crate::domain::sender::sender_entry::SenderEntry;
    use crate::domain::sender::sender_entry_repository::SenderRepositoryError;

    let pool = setup_pool().await;
    let entry1 = SenderEntry::try_from(sample_sender_dto("自宅")).unwrap();
    let entry2 = SenderEntry::try_from(sample_sender_dto("自宅")).unwrap();
    let repo = SqlxSenderEntryRepository::new(pool);

    repo.create(&entry1).await.expect("first create");
    let err = repo
      .create(&entry2)
      .await
      .expect_err("unique index should reject duplicate active label");

    assert!(matches!(err, SenderRepositoryError::DuplicateActiveLabel));
  }

  #[tokio::test]
  async fn create_sender_entry_reuses_label_after_archive() {
    let pool = setup_pool().await;

    create_sender_entry_impl(&pool, sample_sender_dto("自宅"))
      .await
      .expect("first create should succeed");

    let sender_id = fetch_sender_id_by_label(&pool, "自宅").await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    repo
      .archive(&crate::domain::sender::sender_entry::SenderEntryId::from_uuid(
        Uuid::parse_str(&sender_id).unwrap(),
      ))
      .await
      .expect("archive sender");

    create_sender_entry_impl(&pool, sample_sender_dto("自宅"))
      .await
      .expect("archived label should be reusable");
  }

  #[tokio::test]
  async fn update_sender_entry_duplicate_label_is_validation_error() {
    let pool = setup_pool().await;

    create_sender_entry_impl(&pool, sample_sender_dto("A"))
      .await
      .expect("create A");
    create_sender_entry_impl(&pool, sample_sender_dto("B"))
      .await
      .expect("create B");

    let b_id = fetch_sender_id_by_label(&pool, "B").await;
    let err = update_sender_entry_impl(&pool, b_id, sample_sender_dto("A"))
      .await
      .expect_err("should fail due to duplicated label");
    assert!(err.contains("このラベルは既に使用されています"));
  }

  #[tokio::test]
  async fn update_sender_entry_links_sender_not_found_is_validation_error() {
    let pool = setup_pool().await;
    let sender_id = Uuid::new_v4().to_string();
    let err = update_sender_entry_links_impl(&pool, sender_id, vec![])
      .await
      .expect_err("missing sender should fail");
    assert!(err.contains("sender entry not found"));
  }

  #[tokio::test]
  async fn update_sender_entry_links_address_not_found_is_validation_error() {
    let pool = setup_pool().await;
    create_sender_entry_impl(&pool, sample_sender_dto("会社"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "会社").await;

    let bad_address_id = Uuid::new_v4().to_string();
    let err = update_sender_entry_links_impl(&pool, sender_id, vec![bad_address_id])
      .await
      .expect_err("missing address should fail");
    assert!(err.contains("address entry not found"));
  }

  #[tokio::test]
  async fn set_sender_for_address_entry_validates_address_and_sender_existence_and_archived() {
    let pool = setup_pool().await;

    // missing address
    let err = set_sender_for_address_entry_impl(
      &pool,
      Uuid::new_v4().to_string(),
      Some(Uuid::new_v4().to_string()),
    )
    .await
    .expect_err("missing address should fail");
    assert!(err.contains("address entry not found"));

    // existing address, missing sender
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;
    let err = set_sender_for_address_entry_impl(
      &pool,
      address_id.to_string(),
      Some(Uuid::new_v4().to_string()),
    )
    .await
    .expect_err("missing sender should fail");
    assert!(err.contains("sender entry not found"));

    // archived sender
    create_sender_entry_impl(&pool, sample_sender_dto("S"))
      .await
      .expect("create sender S");
    let sender_id = fetch_sender_id_by_label(&pool, "S").await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    repo
      .archive(&crate::domain::sender::sender_entry::SenderEntryId::from_uuid(
        Uuid::parse_str(&sender_id).unwrap(),
      ))
      .await
      .unwrap();

    let err = set_sender_for_address_entry_impl(&pool, address_id.to_string(), Some(sender_id))
      .await
      .expect_err("archived sender should fail");
    assert!(err.contains("sender entry is archived"));

    // archived address
    let archived_address_id = Uuid::new_v4();
    insert_address_entry(&pool, archived_address_id, true).await;
    let err = set_sender_for_address_entry_impl(&pool, archived_address_id.to_string(), None)
      .await
      .expect_err("archived address should fail");
    assert!(err.contains("address entry is archived"));
  }
}

