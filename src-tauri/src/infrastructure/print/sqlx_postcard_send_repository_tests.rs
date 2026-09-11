#[cfg(test)]
mod tests {
  use chrono::NaiveDate;
  use sqlx::SqlitePool;
  use uuid::Uuid;

  use crate::domain::print::postcard_send::PostcardSend;
  use crate::domain::print::postcard_send_repository::{
    Pagination, PostcardSendRepository, PostcardSendRepositoryError, PostcardSendSearchQuery,
    SendStatusFilter, SendStatusQuery, SortOrder,
  };
  use crate::domain::print::postcard_send_source::PostcardSendSource;
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

  async fn seed_address(
    pool: &SqlitePool,
    id: Uuid,
    last: &str,
    first: &str,
    kana_last: Option<&str>,
    kana_first: Option<&str>,
  ) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id, primary_last, primary_first, primary_kana_last, primary_kana_first,
          honorific, postal_code, prefecture, city, street, building, memo, archived_at, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, '様', '1234567', '東京都', '千代田区', '1-1-1', NULL, NULL, NULL, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind(last)
    .bind(first)
    .bind(kana_last)
    .bind(kana_first)
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

  async fn archive_address(pool: &SqlitePool, id: Uuid) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE address_entries SET archived_at = ? WHERE id = ?")
      .bind(&now)
      .bind(id.to_string())
      .execute(pool)
      .await
      .unwrap();
  }

  async fn seed_receipt(pool: &SqlitePool, address_entry_id: Uuid, received_at: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO postcard_receipts (
          id, address_entry_id, sender_display_name, received_at, category, memo,
          deleted_at, created_at, updated_at
        ) VALUES (?, ?, '受取人', ?, 'nenga', NULL, NULL, ?, ?)
      "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(address_entry_id.to_string())
    .bind(received_at)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
  }

  fn sample_send_as_of(
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
    sent_on: NaiveDate,
    postcard_type: PostcardType,
    address_snapshot: &str,
    sender_snapshot: &str,
  ) -> PostcardSend {
    PostcardSend::create_new_as_of(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot.to_string(),
      address_snapshot.to_string(),
      postcard_type,
      sent_on,
      PostcardSendSource::Print,
      None,
      NaiveDate::from_ymd_opt(2099, 12, 31).unwrap(),
    )
    .expect("create_new_as_of")
  }

  fn sample_send(
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
  ) -> PostcardSend {
    sample_send_as_of(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
      PostcardType::Nenga,
      r#"{"address_entry_id":"y","primary_last":"佐藤","primary_first":"一郎"}"#,
      r#"{"sender_entry_id":"x","primary_last":"山田","primary_first":"太郎"}"#,
    )
  }

  fn status_query(
    year: i32,
    status: SendStatusFilter,
    postcard_type: Option<PostcardType>,
    receipt_year: Option<i32>,
    keyword: Option<&str>,
  ) -> SendStatusQuery {
    SendStatusQuery {
      year,
      postcard_type,
      status,
      receipt_year,
      keyword: keyword.map(|s| s.to_string()),
      pagination: Pagination {
        limit: 50,
        offset: 0,
      },
    }
  }

  #[tokio::test]
  async fn create_batch_success() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id, "佐藤", "一郎", None, None).await;
    seed_sender(&pool, sender_id).await;

    let print_job_id = Uuid::new_v4();
    let repo = SqlxPostcardSendRepository::new(pool.clone());
    repo
      .create_batch(&[sample_send(print_job_id, address_id, sender_id)])
      .await
      .expect("create_batch");

    let row = sqlx::query(
      "SELECT source, memo FROM postcard_sends WHERE print_job_id = ? AND deleted_at IS NULL",
    )
    .bind(print_job_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    use sqlx::Row;
    let source: String = row.get("source");
    let memo: Option<String> = row.get("memo");
    assert_eq!(source, "print");
    assert!(memo.is_none());
  }

  #[tokio::test]
  async fn create_batch_duplicate_returns_conflict() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id, "佐藤", "一郎", None, None).await;
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

  #[tokio::test]
  async fn search_send_status_sent_and_unsent() {
    let pool = setup_pool().await;
    let sent_id = Uuid::new_v4();
    let unsent_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, sent_id, "送付", "済", Some("ソウフ"), Some("ズミ")).await;
    seed_address(&pool, unsent_id, "未送", "付", Some("ミソウ"), Some("フ")).await;
    seed_sender(&pool, sender_id).await;

    let repo = SqlxPostcardSendRepository::new(pool.clone());
    repo
      .create_batch(&[sample_send_as_of(
        Uuid::new_v4(),
        sent_id,
        sender_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        PostcardType::Nenga,
        r#"{"primary_last":"送付","primary_first":"済"}"#,
        r#"{"primary_last":"山田","primary_first":"太郎"}"#,
      )])
      .await
      .expect("create");

    let (sent_items, sent_total) = repo
      .search_send_status(status_query(2026, SendStatusFilter::Sent, None, None, None))
      .await
      .expect("sent");
    assert_eq!(sent_total, 1);
    assert_eq!(sent_items[0].address_entry_id, sent_id);

    let (unsent_items, unsent_total) = repo
      .search_send_status(status_query(2026, SendStatusFilter::Unsent, None, None, None))
      .await
      .expect("unsent");
    assert_eq!(unsent_total, 1);
    assert_eq!(unsent_items[0].address_entry_id, unsent_id);
  }

  #[tokio::test]
  async fn search_send_status_receipt_year_independent_of_target_year() {
    let pool = setup_pool().await;
    let with_receipt = Uuid::new_v4();
    let without_receipt = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, with_receipt, "受取", "あり", None, None).await;
    seed_address(&pool, without_receipt, "受取", "なし", None, None).await;
    seed_sender(&pool, sender_id).await;
    seed_receipt(&pool, with_receipt, "2025-12-01").await;

    let repo = SqlxPostcardSendRepository::new(pool);
    let (items, total) = repo
      .search_send_status(status_query(
        2026,
        SendStatusFilter::Unsent,
        None,
        Some(2025),
        None,
      ))
      .await
      .expect("receipt filter");
    assert_eq!(total, 1);
    assert_eq!(items[0].address_entry_id, with_receipt);
  }

  #[tokio::test]
  async fn search_send_status_excludes_archived_address() {
    let pool = setup_pool().await;
    let active_id = Uuid::new_v4();
    let archived_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, active_id, "活性", "宛", None, None).await;
    seed_address(&pool, archived_id, "保管", "宛", None, None).await;
    seed_sender(&pool, sender_id).await;
    archive_address(&pool, archived_id).await;

    let repo = SqlxPostcardSendRepository::new(pool.clone());
    repo
      .create_batch(&[
        sample_send(Uuid::new_v4(), active_id, sender_id),
        sample_send(Uuid::new_v4(), archived_id, sender_id),
      ])
      .await
      .expect("create");

    let (sent, _) = repo
      .search_send_status(status_query(2026, SendStatusFilter::Sent, None, None, None))
      .await
      .expect("sent");
    assert!(sent.iter().all(|i| i.address_entry_id == active_id));

    let (unsent, _) = repo
      .search_send_status(status_query(2026, SendStatusFilter::Unsent, None, None, None))
      .await
      .expect("unsent");
    assert!(unsent.iter().all(|i| i.address_entry_id != archived_id));
  }

  #[tokio::test]
  async fn search_send_status_last_sent_on_outside_year_filter() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id, "過去", "送付", None, None).await;
    seed_sender(&pool, sender_id).await;

    let repo = SqlxPostcardSendRepository::new(pool);
    repo
      .create_batch(&[sample_send_as_of(
        Uuid::new_v4(),
        address_id,
        sender_id,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        PostcardType::Nenga,
        r#"{"primary_last":"過去","primary_first":"送付"}"#,
        r#"{"primary_last":"山田","primary_first":"太郎"}"#,
      )])
      .await
      .expect("create");

    let (items, total) = repo
      .search_send_status(status_query(
        2026,
        SendStatusFilter::Unsent,
        Some(PostcardType::Nenga),
        None,
        None,
      ))
      .await
      .expect("unsent 2026");
    assert_eq!(total, 1);
    assert_eq!(
      items[0].last_sent_on,
      Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap())
    );
    assert_eq!(items[0].send_count, 1);
  }

  #[tokio::test]
  async fn search_send_status_sorts_by_kana() {
    let pool = setup_pool().await;
    let a_id = Uuid::new_v4();
    let b_id = Uuid::new_v4();
    seed_address(&pool, a_id, "漢字後", "名", Some("カナア"), Some("メイ")).await;
    seed_address(&pool, b_id, "漢字先", "名", Some("アイウ"), Some("メイ")).await;

    let repo = SqlxPostcardSendRepository::new(pool);
    let (items, total) = repo
      .search_send_status(status_query(2026, SendStatusFilter::Unsent, None, None, None))
      .await
      .expect("kana sort");
    assert_eq!(total, 2);
    assert_eq!(items[0].address_entry_id, b_id);
    assert_eq!(items[1].address_entry_id, a_id);
  }

  #[tokio::test]
  async fn search_history_keyword_escape_and_snapshot_fallback() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id, "削除前", "宛名", None, None).await;
    seed_sender(&pool, sender_id).await;

    let repo = SqlxPostcardSendRepository::new(pool.clone());
    let snap_last = r#"スナップ%_\特殊"#;
    let address_snapshot = serde_json::json!({
      "primary_last": snap_last,
      "primary_first": "太郎"
    })
    .to_string();
    repo
      .create_batch(&[sample_send_as_of(
        Uuid::new_v4(),
        address_id,
        sender_id,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        PostcardType::Nenga,
        &address_snapshot,
        r#"{"primary_last":"差出","primary_first":"太郎"}"#,
      )])
      .await
      .expect("create");

    // live 行を消して snapshot fallback を強制（同一接続で FK 一時無効）
    {
      let mut conn = pool.acquire().await.unwrap();
      sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .unwrap();
      sqlx::query("DELETE FROM address_entries WHERE id = ?")
        .bind(address_id.to_string())
        .execute(&mut *conn)
        .await
        .unwrap();
      sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    let (items, total) = repo
      .search(PostcardSendSearchQuery {
        keyword: Some(r#"スナップ%_\"#.to_string()),
        year: Some(2026),
        postcard_type: None,
        address_entry_id: None,
        source: None,
        include_deleted: false,
        pagination: Pagination {
          limit: 20,
          offset: 0,
        },
        sort_order: SortOrder::Desc,
      })
      .await
      .expect("search snapshot");
    assert_eq!(total, 1);
    assert_eq!(items[0].send.address_entry_id(), address_id);
  }

  #[tokio::test]
  async fn search_send_status_type_change_moves_between_sent_unsent() {
    let pool = setup_pool().await;
    let address_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    seed_address(&pool, address_id, "種別", "移動", None, None).await;
    seed_sender(&pool, sender_id).await;

    let repo = SqlxPostcardSendRepository::new(pool);
    repo
      .create_batch(&[sample_send_as_of(
        Uuid::new_v4(),
        address_id,
        sender_id,
        NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        PostcardType::Mochu,
        r#"{"primary_last":"種別","primary_first":"移動"}"#,
        r#"{"primary_last":"山田","primary_first":"太郎"}"#,
      )])
      .await
      .expect("create mochu");

    let (nenga_unsent, _) = repo
      .search_send_status(status_query(
        2026,
        SendStatusFilter::Unsent,
        Some(PostcardType::Nenga),
        None,
        None,
      ))
      .await
      .expect("nenga unsent");
    assert!(nenga_unsent.iter().any(|i| i.address_entry_id == address_id));

    let (mochu_sent, _) = repo
      .search_send_status(status_query(
        2026,
        SendStatusFilter::Sent,
        Some(PostcardType::Mochu),
        None,
        None,
      ))
      .await
      .expect("mochu sent");
    assert!(mochu_sent.iter().any(|i| i.address_entry_id == address_id));
  }
}
