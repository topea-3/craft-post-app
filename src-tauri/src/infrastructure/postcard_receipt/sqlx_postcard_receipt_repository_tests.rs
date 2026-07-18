#[cfg(test)]
mod tests {
  use chrono::NaiveDate;
  use sqlx::SqlitePool;
  use uuid::Uuid;

  use crate::domain::address::address::Address;
  use crate::domain::address::address_entry::AddressEntry;
  use crate::domain::address::address_entry_repository::AddressEntryRepository;
  use crate::domain::address::honorific::Honorific;
  use crate::domain::address::person_name::PersonName;
  use crate::domain::address::postal_code::PostalCode;
  use crate::domain::postcard_receipt::postcard_receipt::PostcardReceipt;
  use crate::domain::postcard_receipt::postcard_receipt_category::PostcardReceiptCategory;
  use crate::domain::postcard_receipt::postcard_receipt_repository::{
    Pagination, PostcardReceiptRepository, PostcardReceiptSearchQuery, SortOrder,
  };
  use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;
  use crate::infrastructure::postcard_receipt::sqlx_postcard_receipt_repository::SqlxPostcardReceiptRepository;

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

  async fn seed_address(pool: &SqlitePool, last: &str) -> Uuid {
    let primary = PersonName::new(last.into(), "太郎".into(), None, None).unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new("東京都".into(), "渋谷区".into(), "1-1-1".into(), None).unwrap();
    let entry = AddressEntry::create_new(primary, vec![], Honorific::Sama, postal, addr, None);
    let id = entry.id().as_uuid();
    SqlxAddressEntryRepository::new(pool.clone())
      .create(&entry)
      .await
      .unwrap();
    id
  }

  fn sample_receipt(address_entry_id: Option<Uuid>, sender_name: Option<&str>, date: NaiveDate) -> PostcardReceipt {
    PostcardReceipt::create_new(
      address_entry_id,
      sender_name.map(str::to_string),
      date,
      PostcardReceiptCategory::Nenga,
      None,
    )
    .unwrap()
  }

  #[tokio::test]
  async fn create_and_find_by_id_roundtrip() {
    let pool = setup_pool().await;
    let repo = SqlxPostcardReceiptRepository::new(pool);
    let receipt = sample_receipt(None, Some("田中家"), NaiveDate::from_ymd_opt(2025, 1, 3).unwrap());
    let id = receipt.id().clone();

    repo.create(&receipt).await.unwrap();
    let found = repo.find_by_id(&id).await.unwrap().expect("receipt not found");
    assert_eq!(found.receipt.sender_display_name(), Some("田中家"));
  }

  #[tokio::test]
  async fn delete_excludes_from_active_search() {
    let pool = setup_pool().await;
    let repo = SqlxPostcardReceiptRepository::new(pool.clone());
    let receipt = sample_receipt(None, Some("削除対象"), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    let id = receipt.id().clone();
    repo.create(&receipt).await.unwrap();
    repo.delete(&id).await.unwrap();

    let (items, total) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: None,
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination { limit: 20, offset: 0 },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();

    assert_eq!(total, 0);
    assert!(items.is_empty());
  }

  #[tokio::test]
  async fn search_filters_by_year_category_and_address() {
    let pool = setup_pool().await;
    let address_id = seed_address(&pool, "佐藤").await;
    let repo = SqlxPostcardReceiptRepository::new(pool.clone());

    let linked = sample_receipt(Some(address_id), None, NaiveDate::from_ymd_opt(2025, 1, 10).unwrap());
    let anonymous = sample_receipt(None, Some("匿名"), NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    repo.create(&linked).await.unwrap();
    repo.create(&anonymous).await.unwrap();

    let (items, total) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: None,
        year: Some(2025),
        category: Some(PostcardReceiptCategory::Nenga),
        address_entry_id: Some(address_id),
        include_deleted: false,
        pagination: Pagination { limit: 20, offset: 0 },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();

    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].receipt.address_entry_id(), Some(address_id));
  }

  #[tokio::test]
  async fn search_by_keyword_matches_sender_display_name_and_address_name() {
    let pool = setup_pool().await;
    let address_id = seed_address(&pool, "鈴木").await;
    let repo = SqlxPostcardReceiptRepository::new(pool.clone());

    repo.create(&sample_receipt(
      Some(address_id),
      None,
      NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
    ))
    .await
    .unwrap();
    repo.create(&sample_receipt(
      None,
      Some("山田家"),
      NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(),
    ))
    .await
    .unwrap();

    let (by_address, total_a) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: Some("鈴木".to_string()),
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination { limit: 20, offset: 0 },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();
    assert_eq!(total_a, 1);
    assert_eq!(by_address.len(), 1);

    let (by_sender, total_s) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: Some("山田".to_string()),
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination { limit: 20, offset: 0 },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();
    assert_eq!(total_s, 1);
    assert_eq!(by_sender.len(), 1);
  }

  #[tokio::test]
  async fn search_on_last_page_after_deleting_last_item_returns_empty_items_with_positive_total() {
    // 最終ページの最後の1件を削除したあと、補正前の page/offset のまま検索すると
    // total > 0 でも items が空になる（フロントは clamp して再取得する必要がある）
    let pool = setup_pool().await;
    let repo = SqlxPostcardReceiptRepository::new(pool.clone());
    const PAGE_SIZE: i64 = 2;

    let r1 = sample_receipt(None, Some("A"), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    let r2 = sample_receipt(None, Some("B"), NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
    let r3 = sample_receipt(None, Some("C"), NaiveDate::from_ymd_opt(2025, 1, 3).unwrap());
    // sort desc では page2 (offset=2) に最も古い A が入る
    let last_page_id = r1.id().clone();
    repo.create(&r1).await.unwrap();
    repo.create(&r2).await.unwrap();
    repo.create(&r3).await.unwrap();

    let (page2_before, total_before) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: None,
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination {
          limit: PAGE_SIZE,
          offset: PAGE_SIZE,
        },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();
    assert_eq!(total_before, 3);
    assert_eq!(page2_before.len(), 1);

    repo.delete(&last_page_id).await.unwrap();

    let (page2_after, total_after) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: None,
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination {
          limit: PAGE_SIZE,
          offset: PAGE_SIZE, // 補正前の page=2
        },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();
    assert_eq!(total_after, 2);
    assert!(
      page2_after.is_empty(),
      "stale last-page offset yields empty items while total > 0"
    );

    let clamped_offset = 0; // clampPage(2, 2, 2) => page 1
    let (page1, total_clamped) = repo
      .search(PostcardReceiptSearchQuery {
        keyword: None,
        year: None,
        category: None,
        address_entry_id: None,
        include_deleted: false,
        pagination: Pagination {
          limit: PAGE_SIZE,
          offset: clamped_offset,
        },
        sort_order: SortOrder::Desc,
      })
      .await
      .unwrap();
    assert_eq!(total_clamped, 2);
    assert_eq!(page1.len(), 2);
  }
}
