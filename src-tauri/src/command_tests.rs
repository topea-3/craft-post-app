#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;
  use sqlx::Row;
  use uuid::Uuid;

  use crate::domain::sender::sender_entry_repository::SenderEntryRepository;
  use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

  use crate::{
    create_postcard_receipt_impl, create_sender_entry_impl, delete_postcard_receipt_impl,
    get_postcard_receipt_impl, list_sender_linked_addresses_impl, set_sender_for_address_entry_impl,
    update_postcard_receipt_impl, update_sender_entry_impl, update_sender_entry_links_impl,
    AddressDto, PersonNameDto, PostcardReceiptDtoInput, SenderEntryDtoInput,
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
  async fn list_sender_linked_addresses_excludes_archived_addresses() {
    let pool = setup_pool().await;
    create_sender_entry_impl(&pool, sample_sender_dto("差出人"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "差出人").await;

    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;
    update_sender_entry_links_impl(&pool, sender_id.clone(), vec![address_id.to_string()])
      .await
      .expect("link address");

    let linked = list_sender_linked_addresses_impl(&pool, sender_id.clone())
      .await
      .expect("list linked addresses");
    assert_eq!(linked.len(), 1);

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE address_entries SET archived_at = ? WHERE id = ?")
      .bind(&now)
      .bind(address_id.to_string())
      .execute(&pool)
      .await
      .unwrap();

    let linked = list_sender_linked_addresses_impl(&pool, sender_id)
      .await
      .expect("list linked addresses after archive");
    assert!(linked.is_empty());
  }

  #[tokio::test]
  async fn list_sender_linked_addresses_validates_sender_existence_and_archived() {
    let pool = setup_pool().await;

    let err = list_sender_linked_addresses_impl(&pool, Uuid::new_v4().to_string())
      .await
      .expect_err("missing sender should fail");
    assert!(err.contains("sender entry not found"));

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

    let err = list_sender_linked_addresses_impl(&pool, sender_id)
      .await
      .expect_err("archived sender should fail");
    assert!(err.contains("sender entry is archived"));
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

  fn sample_receipt_dto(
    received_at: &str,
    address_entry_id: Option<String>,
    sender_display_name: Option<String>,
  ) -> PostcardReceiptDtoInput {
    PostcardReceiptDtoInput {
      address_entry_id,
      sender_display_name,
      received_at: received_at.to_string(),
      category: "nenga".to_string(),
      memo: None,
    }
  }

  #[tokio::test]
  async fn create_postcard_receipt_requires_sender_display_name_when_unlinked() {
    let pool = setup_pool().await;
    let err = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", None, None),
    )
    .await
    .expect_err("unlinked without display name should fail");
    assert!(err.contains("送り主の表示名を入力してください"));
  }

  #[tokio::test]
  async fn create_postcard_receipt_rejects_future_received_date() {
    let pool = setup_pool().await;
    let future = (chrono::Utc::now().date_naive() + chrono::Duration::days(2))
      .format("%Y-%m-%d")
      .to_string();
    let err = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(&future, None, Some("田中家".to_string())),
    )
    .await
    .expect_err("future date should fail");
    assert!(err.contains("受取日に未来の日付は指定できません"));
  }

  #[tokio::test]
  async fn create_postcard_receipt_rejects_archived_address_entry() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, true).await;

    let err = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", Some(address_id.to_string()), None),
    )
    .await
    .expect_err("archived address should fail");
    assert!(err.contains("address entry is archived"));
  }

  #[tokio::test]
  async fn update_postcard_receipt_rejects_missing_address_entry() {
    let pool = setup_pool().await;
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", None, Some("田中家".to_string())),
    )
    .await
    .expect("create receipt");

    let err = update_postcard_receipt_impl(
      &pool,
      id,
      sample_receipt_dto("2025-01-03", Some(Uuid::new_v4().to_string()), None),
    )
    .await
    .expect_err("missing address should fail");
    assert!(err.contains("address entry not found"));
  }

  #[tokio::test]
  async fn update_postcard_receipt_allows_existing_archived_address_entry() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;

    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", Some(address_id.to_string()), None),
    )
    .await
    .expect("create receipt");

    sqlx::query("UPDATE address_entries SET archived_at = ? WHERE id = ?")
      .bind(chrono::Utc::now().to_rfc3339())
      .bind(address_id.to_string())
      .execute(&pool)
      .await
      .expect("archive address");

    let mut dto = sample_receipt_dto("2025-01-04", Some(address_id.to_string()), None);
    dto.memo = Some("メモ更新".to_string());
    update_postcard_receipt_impl(&pool, id.clone(), dto)
      .await
      .expect("update with same archived address should succeed");

    let got = get_postcard_receipt_impl(&pool, id)
      .await
      .expect("get receipt");
    assert_eq!(got.received_at, "2025-01-04");
    assert_eq!(got.memo.as_deref(), Some("メモ更新"));
    assert_eq!(got.address_entry_id.as_deref(), Some(address_id.to_string().as_str()));
    assert_eq!(got.address_entry_archived, Some(true));
  }

  #[tokio::test]
  async fn update_postcard_receipt_rejects_switching_to_archived_address_entry() {
    let pool = setup_pool().await;
    let active_id = Uuid::new_v4();
    let archived_id = Uuid::new_v4();
    insert_address_entry(&pool, active_id, false).await;
    insert_address_entry(&pool, archived_id, true).await;

    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", Some(active_id.to_string()), None),
    )
    .await
    .expect("create receipt");

    let err = update_postcard_receipt_impl(
      &pool,
      id,
      sample_receipt_dto("2025-01-03", Some(archived_id.to_string()), None),
    )
    .await
    .expect_err("switching to archived address should fail");
    assert!(err.contains("address entry is archived"));
  }

  #[tokio::test]
  async fn delete_postcard_receipt_makes_get_fail() {
    let pool = setup_pool().await;
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", None, Some("削除テスト".to_string())),
    )
    .await
    .expect("create receipt");

    delete_postcard_receipt_impl(&pool, id.clone())
      .await
      .expect("delete receipt");

    let err = get_postcard_receipt_impl(&pool, id)
      .await
      .expect_err("deleted receipt should not be returned");
    assert!(err.contains("postcard receipt not found"));
  }
}

