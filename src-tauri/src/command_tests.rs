#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;
  use sqlx::Row;
  use uuid::Uuid;

  use crate::domain::sender::sender_entry_repository::SenderEntryRepository;
  use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

  use crate::{
    build_address_print_snapshot_impl, build_sender_print_snapshot_impl,
    create_postcard_receipt_impl, create_postcard_sends_batch_impl, create_sender_entry_impl,
    delete_postcard_receipt_impl, filter_active_address_entry_ids_impl, get_postcard_receipt_impl,
    list_print_layout_preferences_impl, list_sender_linked_addresses_impl,
    resolve_print_job_items_impl, save_print_layout_preferences_impl,
    search_postcard_receipts_impl, set_sender_for_address_entry_impl, update_postcard_receipt_impl,
    update_sender_entry_impl, update_sender_entry_links_impl, AddressDto,
    AddressPrintSnapshotDto, CreatePostcardSendItemDto, CreatePostcardSendsBatchInput,
    PersonNameDto, PostcardReceiptDtoInput, PrintLayoutPreferenceDto, SenderEntryDtoInput,
    SenderPrintSnapshotDto,
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

  async fn update_receipt(
    pool: &SqlitePool,
    id: String,
    dto: PostcardReceiptDtoInput,
  ) -> Result<(), String> {
    let current = get_postcard_receipt_impl(pool, id.clone()).await?;
    update_postcard_receipt_impl(pool, id, dto, current.updated_at).await
  }

  // postcard command rejection 文字列（フロントの KNOWN_ERROR_MESSAGES と一致させる）
  const RECEIPT_FUTURE_DATE_MESSAGE: &str = "受取日に未来の日付は指定できません。";
  const RECEIPT_SENDER_DISPLAY_NAME_REQUIRED_MESSAGE: &str = "送り主の表示名を入力してください。";
  const RECEIPT_NOT_FOUND_MESSAGE: &str = "postcard receipt not found";
  const RECEIPT_CONFLICT_MESSAGE: &str =
    "他の操作で更新済みです。画面を再読み込みしてから再度保存してください。";
  const ADDRESS_ENTRY_NOT_FOUND_MESSAGE: &str = "address entry not found";
  const ADDRESS_ENTRY_ARCHIVED_MESSAGE: &str = "address entry is archived";

  #[tokio::test]
  async fn create_postcard_receipt_requires_sender_display_name_when_unlinked() {
    let pool = setup_pool().await;
    let err = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", None, None),
    )
    .await
    .expect_err("unlinked without display name should fail");
    assert_eq!(err, RECEIPT_SENDER_DISPLAY_NAME_REQUIRED_MESSAGE);
  }

  #[tokio::test]
  async fn create_postcard_receipt_allows_local_today() {
    let pool = setup_pool().await;
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(&today, None, Some("田中家".to_string())),
    )
    .await
    .expect("local today must be allowed");
    let got = get_postcard_receipt_impl(&pool, id).await.expect("get receipt");
    assert_eq!(got.received_at, today);
  }

  #[tokio::test]
  async fn create_postcard_receipt_rejects_future_received_date() {
    let pool = setup_pool().await;
    // 日付跨ぎフレーク回避のため十分遠い固定未来日を使う
    let far_future = "2099-12-31";
    let err = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(far_future, None, Some("田中家".to_string())),
    )
    .await
    .expect_err("far future date should fail");
    assert_eq!(err, RECEIPT_FUTURE_DATE_MESSAGE);
  }

  #[tokio::test]
  async fn update_postcard_receipt_rejects_local_tomorrow() {
    let pool = setup_pool().await;
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(&today, None, Some("田中家".to_string())),
    )
    .await
    .expect("create receipt");

    let far_future = "2099-12-31";
    let err = update_receipt(
      &pool,
      id,
      sample_receipt_dto(far_future, None, Some("田中家".to_string())),
    )
    .await
    .expect_err("update with far future date should fail");
    assert_eq!(err, RECEIPT_FUTURE_DATE_MESSAGE);
  }

  #[tokio::test]
  async fn update_postcard_receipt_allows_memo_change_when_received_at_looks_future() {
    let pool = setup_pool().await;
    // DB 直挿入の「未来に見える日」も固定遠未来で境界フレークを避ける
    let future_looking = "2099-06-01";
    let further_future = "2099-12-31";
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO postcard_receipts (
          id, address_entry_id, sender_display_name, received_at, category, memo,
          deleted_at, created_at, updated_at
        ) VALUES (?, NULL, ?, ?, 'nenga', NULL, NULL, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind("未来見え")
    .bind(future_looking)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let mut dto = sample_receipt_dto(future_looking, None, Some("未来見え".to_string()));
    dto.memo = Some("メモのみ".to_string());
    update_receipt(&pool, id.to_string(), dto)
      .await
      .expect("memo-only update with unchanged future-looking date must succeed");

    let err = update_receipt(
      &pool,
      id.to_string(),
      sample_receipt_dto(further_future, None, Some("未来見え".to_string())),
    )
    .await
    .expect_err("moving further into the future must fail");
    assert_eq!(err, RECEIPT_FUTURE_DATE_MESSAGE);
  }

  #[tokio::test]
  async fn search_postcard_receipts_excludes_deleted_rows() {
    let pool = setup_pool().await;
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto("2025-01-03", None, Some("検索対象".to_string())),
    )
    .await
    .expect("create");

    let before = search_postcard_receipts_impl(
      &pool,
      Some("検索対象".to_string()),
      None,
      None,
      None,
      Some(20),
      Some(0),
      None,
    )
    .await
    .expect("search before delete");
    assert_eq!(before.total, 1);

    delete_postcard_receipt_impl(&pool, id)
      .await
      .expect("delete");

    let after = search_postcard_receipts_impl(
      &pool,
      Some("検索対象".to_string()),
      None,
      None,
      None,
      Some(20),
      Some(0),
      None,
    )
    .await
    .expect("search after delete");
    assert_eq!(after.total, 0);
    assert!(after.items.is_empty());
  }

  #[tokio::test]
  async fn update_postcard_receipt_rejects_stale_expected_updated_at() {
    let pool = setup_pool().await;
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(&today, None, Some("同時編集".to_string())),
    )
    .await
    .expect("create receipt");

    let loaded = get_postcard_receipt_impl(&pool, id.clone())
      .await
      .expect("get receipt");
    let stale_updated_at = loaded.updated_at.clone();

    let mut first = sample_receipt_dto(&today, None, Some("同時編集".to_string()));
    first.category = "mochu".to_string();
    update_postcard_receipt_impl(&pool, id.clone(), first, stale_updated_at.clone())
      .await
      .expect("first update succeeds");

    let mut second = sample_receipt_dto(&today, None, Some("同時編集".to_string()));
    second.memo = Some("後勝ちメモ".to_string());
    let err = update_postcard_receipt_impl(&pool, id.clone(), second, stale_updated_at)
      .await
      .expect_err("stale update must conflict");
    assert_eq!(err, RECEIPT_CONFLICT_MESSAGE);

    let got = get_postcard_receipt_impl(&pool, id).await.expect("get receipt");
    assert_eq!(got.category, "mochu");
    assert!(got.memo.is_none());
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
    assert_eq!(err, ADDRESS_ENTRY_ARCHIVED_MESSAGE);
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

    let err = update_receipt(
      &pool,
      id,
      sample_receipt_dto("2025-01-03", Some(Uuid::new_v4().to_string()), None),
    )
    .await
    .expect_err("missing address should fail");
    assert_eq!(err, ADDRESS_ENTRY_NOT_FOUND_MESSAGE);
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
    update_receipt(&pool, id.clone(), dto)
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

    let err = update_receipt(
      &pool,
      id,
      sample_receipt_dto("2025-01-03", Some(archived_id.to_string()), None),
    )
    .await
    .expect_err("switching to archived address should fail");
    assert_eq!(err, ADDRESS_ENTRY_ARCHIVED_MESSAGE);
  }

  #[tokio::test]
  async fn update_postcard_receipt_rejects_after_delete() {
    let pool = setup_pool().await;
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let id = create_postcard_receipt_impl(
      &pool,
      sample_receipt_dto(&today, None, Some("削除競合".to_string())),
    )
    .await
    .expect("create receipt");

    delete_postcard_receipt_impl(&pool, id.clone())
      .await
      .expect("delete receipt");

    let err = update_postcard_receipt_impl(
      &pool,
      id,
      sample_receipt_dto(&today, None, Some("更新しようとする".to_string())),
      chrono::Utc::now().to_rfc3339(),
    )
    .await
    .expect_err("update after delete should fail");
    assert_eq!(err, RECEIPT_NOT_FOUND_MESSAGE);
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
    assert_eq!(err, RECEIPT_NOT_FOUND_MESSAGE);
  }

  // ---------------------------------------------------------------------------
  // Print (TOP-28)
  // ---------------------------------------------------------------------------

  fn sample_address_snapshot(address_entry_id: &str) -> AddressPrintSnapshotDto {
    AddressPrintSnapshotDto {
      address_entry_id: address_entry_id.to_string(),
      postal_code: "1234567".to_string(),
      address_line1: "東京都千代田区".to_string(),
      address_line2: "1-1-1".to_string(),
      address_line3: String::new(),
      primary_last: "佐藤".to_string(),
      primary_first: "一郎".to_string(),
      co_recipients: vec![],
      honorific_print: "様".to_string(),
    }
  }

  fn sample_sender_snapshot(sender_entry_id: &str) -> SenderPrintSnapshotDto {
    SenderPrintSnapshotDto {
      sender_entry_id: sender_entry_id.to_string(),
      postal_code: "1234567".to_string(),
      address_line1: "東京都渋谷区".to_string(),
      address_line2: "神南 1-1-1".to_string(),
      address_line3: String::new(),
      primary_last: "山田".to_string(),
      primary_first: "太郎".to_string(),
      co_recipients: vec![],
    }
  }

  #[tokio::test]
  async fn filter_active_address_entry_ids_keeps_order_and_drops_archived_invalid() {
    let pool = setup_pool().await;
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let id_c = Uuid::new_v4();
    insert_address_entry(&pool, id_a, false).await;
    insert_address_entry(&pool, id_b, false).await;
    insert_address_entry(&pool, id_c, true).await;

    let result = filter_active_address_entry_ids_impl(
      &pool,
      vec![
        id_b.to_string(),
        "not-a-uuid".to_string(),
        id_c.to_string(),
        id_a.to_string(),
      ],
    )
    .await
    .expect("filter should succeed");

    assert_eq!(result, vec![id_b.to_string(), id_a.to_string()]);
  }

  #[tokio::test]
  async fn filter_active_address_entry_ids_empty_ok() {
    let pool = setup_pool().await;
    let result = filter_active_address_entry_ids_impl(&pool, vec![])
      .await
      .expect("empty input should succeed");
    assert!(result.is_empty());
  }

  #[tokio::test]
  async fn resolve_print_job_items_success_with_linked_sender() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;
    create_sender_entry_impl(&pool, sample_sender_dto("印刷差出人"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "印刷差出人").await;
    set_sender_for_address_entry_impl(&pool, address_id.to_string(), Some(sender_id))
      .await
      .expect("link sender");

    let result = resolve_print_job_items_impl(&pool, vec![address_id.to_string()])
      .await
      .expect("resolve should succeed");

    assert_eq!(result.items.len(), 1);
    assert!(result.excluded.is_empty());
    assert_eq!(result.items[0].address.address_entry_id, address_id.to_string());
  }

  #[tokio::test]
  async fn resolve_print_job_items_excludes_unlinked_and_archived_sender() {
    let pool = setup_pool().await;

    let unlinked_id = Uuid::new_v4();
    insert_address_entry(&pool, unlinked_id, false).await;

    let archived_link_id = Uuid::new_v4();
    insert_address_entry(&pool, archived_link_id, false).await;
    create_sender_entry_impl(&pool, sample_sender_dto("アーカイブ差出人"))
      .await
      .expect("create archived sender");
    let archived_sender_id = fetch_sender_id_by_label(&pool, "アーカイブ差出人").await;
    set_sender_for_address_entry_impl(
      &pool,
      archived_link_id.to_string(),
      Some(archived_sender_id.clone()),
    )
    .await
    .expect("link archived sender");
    // リポジトリ archive はリンク削除するため、リンクを残したまま SQL で archived にする
    sqlx::query("UPDATE sender_entries SET archived_at = ? WHERE id = ?")
      .bind(chrono::Utc::now().to_rfc3339())
      .bind(&archived_sender_id)
      .execute(&pool)
      .await
      .unwrap();

    let good_id = Uuid::new_v4();
    insert_address_entry(&pool, good_id, false).await;
    create_sender_entry_impl(&pool, sample_sender_dto("有効差出人"))
      .await
      .expect("create good sender");
    let good_sender_id = fetch_sender_id_by_label(&pool, "有効差出人").await;
    set_sender_for_address_entry_impl(&pool, good_id.to_string(), Some(good_sender_id))
      .await
      .expect("link good sender");

    let result = resolve_print_job_items_impl(
      &pool,
      vec![
        unlinked_id.to_string(),
        archived_link_id.to_string(),
        good_id.to_string(),
      ],
    )
    .await
    .expect("resolve should succeed with exclusions");

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].address.address_entry_id, good_id.to_string());
    assert_eq!(result.excluded.len(), 2);
    assert_eq!(result.excluded[0].address_entry_id, unlinked_id.to_string());
    assert_eq!(result.excluded[0].reason, "no_sender_link");
    assert_eq!(
      result.excluded[1].address_entry_id,
      archived_link_id.to_string()
    );
    assert_eq!(result.excluded[1].reason, "sender_archived");
  }

  #[tokio::test]
  async fn resolve_print_job_items_fails_on_archived_address_with_json() {
    let pool = setup_pool().await;
    let archived_id = Uuid::new_v4();
    insert_address_entry(&pool, archived_id, true).await;

    let err = resolve_print_job_items_impl(&pool, vec![archived_id.to_string()])
      .await
      .expect_err("archived address should fail");

    assert!(err.contains("ADDRESS_ENTRIES_INVALID"));
    assert!(err.contains("archived"));
  }

  #[tokio::test]
  async fn resolve_print_job_items_fails_on_not_found() {
    let pool = setup_pool().await;
    let missing_id = Uuid::new_v4().to_string();

    let err = resolve_print_job_items_impl(&pool, vec![missing_id])
      .await
      .expect_err("missing address should fail");

    assert!(err.contains("ADDRESS_ENTRIES_INVALID") || err.contains("not_found"));
    assert!(err.contains("not_found"));
  }

  #[tokio::test]
  async fn build_address_print_snapshot_errors_when_archived() {
    let pool = setup_pool().await;
    let archived_id = Uuid::new_v4();
    insert_address_entry(&pool, archived_id, true).await;

    let err = build_address_print_snapshot_impl(&pool, archived_id.to_string())
      .await
      .expect_err("archived address snapshot should fail");
    assert!(err.contains("archived"));
  }

  #[tokio::test]
  async fn build_sender_print_snapshot_errors_when_archived() {
    let pool = setup_pool().await;
    create_sender_entry_impl(&pool, sample_sender_dto("スナップショット差出人"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "スナップショット差出人").await;
    sqlx::query("UPDATE sender_entries SET archived_at = ? WHERE id = ?")
      .bind(chrono::Utc::now().to_rfc3339())
      .bind(&sender_id)
      .execute(&pool)
      .await
      .unwrap();

    let err = build_sender_print_snapshot_impl(&pool, sender_id)
      .await
      .expect_err("archived sender snapshot should fail");
    assert!(err.contains("archived"));
  }

  #[tokio::test]
  async fn save_and_list_print_layout_preferences_roundtrip() {
    let pool = setup_pool().await;
    let offsets = vec![
      PrintLayoutPreferenceDto {
        layer_id: "recipient.postalCode".to_string(),
        offset_x_pt: 1.5,
        offset_y_pt: -2.0,
      },
      PrintLayoutPreferenceDto {
        layer_id: "sender.primaryLast".to_string(),
        offset_x_pt: 3.0,
        offset_y_pt: 4.25,
      },
    ];

    save_print_layout_preferences_impl(&pool, "nenga".to_string(), offsets.clone())
      .await
      .expect("save prefs");

    let listed = list_print_layout_preferences_impl(&pool, "nenga".to_string())
      .await
      .expect("list prefs");

    assert_eq!(listed.len(), 2);
    let postal = listed
      .iter()
      .find(|p| p.layer_id == "recipient.postalCode")
      .expect("postal layer");
    assert_eq!(postal.offset_x_pt, 1.5);
    assert_eq!(postal.offset_y_pt, -2.0);
    let sender_last = listed
      .iter()
      .find(|p| p.layer_id == "sender.primaryLast")
      .expect("sender last layer");
    assert_eq!(sender_last.offset_x_pt, 3.0);
    assert_eq!(sender_last.offset_y_pt, 4.25);

    let mochu = list_print_layout_preferences_impl(&pool, "mochu".to_string())
      .await
      .expect("list mochu prefs");
    assert!(mochu.is_empty());
  }

  #[tokio::test]
  async fn create_postcard_sends_batch_inserts_and_idempotent_on_retry() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;
    create_sender_entry_impl(&pool, sample_sender_dto("送付バッチ差出人"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "送付バッチ差出人").await;

    let print_job_id = Uuid::new_v4().to_string();
    let input = CreatePostcardSendsBatchInput {
      print_job_id: print_job_id.clone(),
      postcard_type: "nenga".to_string(),
      items: vec![CreatePostcardSendItemDto {
        address_entry_id: address_id.to_string(),
        sender_entry_id: sender_id.clone(),
        address_snapshot: sample_address_snapshot(&address_id.to_string()),
        sender_snapshot: sample_sender_snapshot(&sender_id),
      }],
    };

    create_postcard_sends_batch_impl(&pool, input.clone())
      .await
      .expect("first batch create");
    create_postcard_sends_batch_impl(&pool, input)
      .await
      .expect("retry should be idempotent Ok");

    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM postcard_sends WHERE print_job_id = ? AND deleted_at IS NULL",
    )
    .bind(&print_job_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);

    let sent_on: String =
      sqlx::query_scalar("SELECT sent_on FROM postcard_sends WHERE print_job_id = ?")
        .bind(&print_job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    assert_eq!(sent_on, today);
  }

  #[tokio::test]
  async fn create_postcard_sends_batch_dedupes_duplicate_address_in_same_batch() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id, false).await;
    create_sender_entry_impl(&pool, sample_sender_dto("送付重複差出人"))
      .await
      .expect("create sender");
    let sender_id = fetch_sender_id_by_label(&pool, "送付重複差出人").await;

    let print_job_id = Uuid::new_v4().to_string();
    let item = CreatePostcardSendItemDto {
      address_entry_id: address_id.to_string(),
      sender_entry_id: sender_id.clone(),
      address_snapshot: sample_address_snapshot(&address_id.to_string()),
      sender_snapshot: sample_sender_snapshot(&sender_id),
    };
    let input = CreatePostcardSendsBatchInput {
      print_job_id: print_job_id.clone(),
      postcard_type: "nenga".to_string(),
      items: vec![item.clone(), item],
    };

    create_postcard_sends_batch_impl(&pool, input)
      .await
      .expect("duplicate address in batch should succeed after dedupe");

    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM postcard_sends WHERE print_job_id = ? AND deleted_at IS NULL",
    )
    .bind(&print_job_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
  }

  #[tokio::test]
  async fn save_print_layout_preferences_rejects_unknown_layer() {
    let pool = setup_pool().await;
    let offsets = vec![PrintLayoutPreferenceDto {
      layer_id: "not.a.real.layer".to_string(),
      offset_x_pt: 1.0,
      offset_y_pt: 2.0,
    }];
    let err = save_print_layout_preferences_impl(&pool, "nenga".to_string(), offsets)
      .await
      .expect_err("unknown layer should fail");
    assert!(err.contains("unknown layer_id") || err.contains("not.a.real.layer"));
  }

  #[tokio::test]
  async fn save_print_layout_preferences_rejects_non_finite_offset() {
    let pool = setup_pool().await;
    let offsets = vec![PrintLayoutPreferenceDto {
      layer_id: "recipient.postalCode".to_string(),
      offset_x_pt: f64::NAN,
      offset_y_pt: 0.0,
    }];
    let err = save_print_layout_preferences_impl(&pool, "nenga".to_string(), offsets)
      .await
      .expect_err("NaN offset should fail");
    assert!(err.contains("finite"));
  }
}

