#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;

  use crate::domain::address::address::Address;
  use crate::domain::address::address_entry::AddressEntry;
  use crate::domain::address::address_entry_repository::{
    AddressEntryRepository, AddressSearchQuery, Pagination, SortKey, SortOrder,
  };
  use crate::domain::address::honorific::Honorific;
  use crate::domain::address::memo::Memo;
  use crate::domain::address::person_name::PersonName;
  use crate::domain::address::postal_code::PostalCode;
  use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;

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

  fn sample_entry() -> AddressEntry {
    let primary = PersonName::new(
      "山田".into(),
      "太郎".into(),
      Some("ヤマダ".into()),
      Some("タロウ".into()),
    )
    .unwrap();
    let co = PersonName::new(
      "山田".into(),
      "花子".into(),
      Some("ヤマダ".into()),
      Some("ハナコ".into()),
    )
    .unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new(
      "東京都".into(),
      "渋谷区".into(),
      "神南 1-1-1".into(),
      Some("○○ビル 3F".into()),
    )
    .unwrap();
    let memo = Some(Memo::new("テストメモ").unwrap());

    AddressEntry::create_new(primary, vec![co], Honorific::Sama, postal, addr, memo, true)
  }

  #[tokio::test]
  async fn create_and_find_by_id_roundtrip() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool);

    let entry = sample_entry();
    let id = entry.id().clone();

    repo.create(&entry).await.unwrap();

    let found = repo.find_by_id(&id).await.unwrap().expect("entry not found");
    assert_eq!(found.id(), &id);
    assert_eq!(found.primary_name().display(), entry.primary_name().display());
    assert_eq!(
      found.display_full_recipient(),
      entry.display_full_recipient()
    );
  }

  #[tokio::test]
  async fn archive_excludes_from_list_active() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool.clone());

    let entry = sample_entry();
    let id = entry.id().clone();

    repo.create(&entry).await.unwrap();

    let list = repo
      .list_active(Pagination { limit: 10, offset: 0 })
      .await
      .unwrap();
    assert_eq!(list.len(), 1);

    repo.archive(&id).await.unwrap();

    let list_after = repo
      .list_active(Pagination { limit: 10, offset: 0 })
      .await
      .unwrap();
    assert!(list_after.is_empty());
  }

  #[tokio::test]
  async fn search_by_keyword_and_sort() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool);

    let entry = sample_entry();
    repo.create(&entry).await.unwrap();

    let query = AddressSearchQuery {
      keyword: Some("山田".into()),
      sort_key: SortKey::NameKana,
      sort_order: SortOrder::Asc,
      include_archived: false,
      pagination: Some(Pagination { limit: 10, offset: 0 }),
    };

    let (entries, total) = repo.search(query).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(total, 1);
  }

  #[tokio::test]
  async fn search_keyword_escapes_like_wildcards() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool);

    let primary = PersonName::new(
      "山田".into(),
      "太郎".into(),
      Some("ヤマダ".into()),
      Some("タロウ".into()),
    )
    .unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new(
      "東京都".into(),
      "渋谷区".into(),
      "神南 1-1-1".into(),
      None,
    )
    .unwrap();
    let memo = Some(Memo::new("100%保証_メモ").unwrap());
    let with_wildcards = AddressEntry::create_new(primary, vec![], Honorific::Sama, postal, addr, memo, true);
    repo.create(&with_wildcards).await.unwrap();

    // エスケープ無しだと `%` / `_` キーワードでヒットしてしまう対照行
    let control_primary = PersonName::new(
      "佐藤".into(),
      "次郎".into(),
      Some("サトウ".into()),
      Some("ジロウ".into()),
    )
    .unwrap();
    let control_postal = PostalCode::new("9876543").unwrap();
    let control_addr = Address::new(
      "大阪府".into(),
      "大阪市".into(),
      "梅田 2-2-2".into(),
      None,
    )
    .unwrap();
    let control = AddressEntry::create_new(
      control_primary,
      vec![],
      Honorific::Sama,
      control_postal,
      control_addr,
      None,
      true,
    );
    repo.create(&control).await.unwrap();

    let wildcard_query = AddressSearchQuery {
      keyword: Some("%".into()),
      sort_key: SortKey::UpdatedAt,
      sort_order: SortOrder::Desc,
      include_archived: false,
      pagination: Some(Pagination { limit: 10, offset: 0 }),
    };
    let (wildcard_hits, wildcard_total) = repo.search(wildcard_query).await.unwrap();
    assert_eq!(wildcard_hits.len(), 1);
    assert_eq!(wildcard_total, 1);
    assert_eq!(wildcard_hits[0].primary_name().last(), "山田");

    let underscore_only = AddressSearchQuery {
      keyword: Some("_".into()),
      sort_key: SortKey::UpdatedAt,
      sort_order: SortOrder::Desc,
      include_archived: false,
      pagination: Some(Pagination { limit: 10, offset: 0 }),
    };
    let (underscore_hits, underscore_total) = repo.search(underscore_only).await.unwrap();
    assert_eq!(underscore_hits.len(), 1);
    assert_eq!(underscore_total, 1);
    assert_eq!(underscore_hits[0].primary_name().last(), "山田");

    let no_match = AddressSearchQuery {
      keyword: Some("存在しない%キーワード".into()),
      sort_key: SortKey::UpdatedAt,
      sort_order: SortOrder::Desc,
      include_archived: false,
      pagination: Some(Pagination { limit: 10, offset: 0 }),
    };
    let (empty, empty_total) = repo.search(no_match).await.unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty_total, 0);
  }

  fn named_entry(last: &str, kana_last: Option<&str>, nenga_send_target: bool) -> AddressEntry {
    let primary = PersonName::new(
      last.into(),
      "太郎".into(),
      kana_last.map(|s| s.to_string()),
      Some("タロウ".into()),
    )
    .unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new("東京都".into(), "千代田区".into(), "1-1".into(), None).unwrap();
    AddressEntry::create_new(primary, vec![], Honorific::Sama, postal, addr, None, nenga_send_target)
  }

  async fn insert_receipt(
    pool: &SqlitePool,
    address_id: Option<&str>,
    category: &str,
    receipt_year: i32,
    deleted: bool,
  ) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO postcard_receipts (
          id, address_entry_id, sender_display_name, received_at, category, memo,
          deleted_at, created_at, updated_at, receipt_year
        ) VALUES (?, ?, NULL, ?, ?, NULL, ?, ?, ?, ?)
      "#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(address_id)
    .bind(format!("{receipt_year}-01-03"))
    .bind(category)
    .bind(if deleted { Some(now.as_str()) } else { None })
    .bind(&now)
    .bind(&now)
    .bind(receipt_year)
    .execute(pool)
    .await
    .unwrap();
  }

  #[tokio::test]
  async fn omitted_column_defaults_nenga_send_target_to_true() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool.clone());
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id, primary_last, primary_first, honorific, postal_code,
          prefecture, city, street, created_at, updated_at
        ) VALUES (?, '省略', '太郎', '様', '1234567', '東京都', '千代田区', '1-1', ?, ?)
      "#,
    )
    .bind(&id)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let found = repo
      .find_by_id(&crate::domain::address::address_entry::AddressEntryId::from_uuid(
        uuid::Uuid::parse_str(&id).unwrap(),
      ))
      .await
      .unwrap()
      .unwrap();
    assert!(found.nenga_send_target());
  }

  #[tokio::test]
  async fn list_nenga_target_review_filters_receipts_archive_and_flag() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool.clone());

    let candidate = named_entry("山田", Some("ヤマダ"), true);
    let got_nenga = named_entry("佐藤", Some("サトウ"), true);
    let got_mochu = named_entry("高橋", Some("タカハシ"), true);
    let got_other = named_entry("伊藤", Some("イトウ"), true);
    let deleted_nenga = named_entry("渡辺", Some("ワタナベ"), true);
    let already_off = named_entry("小林", Some("コバヤシ"), false);
    let archived = named_entry("加藤", Some("カトウ"), true);
    for entry in [
      &candidate,
      &got_nenga,
      &got_mochu,
      &got_other,
      &deleted_nenga,
      &already_off,
      &archived,
    ] {
      repo.create(entry).await.unwrap();
    }
    repo.archive(archived.id()).await.unwrap();

    let year = 2026;
    insert_receipt(&pool, Some(&got_nenga.id().as_uuid().to_string()), "nenga", year, false).await;
    insert_receipt(&pool, Some(&got_mochu.id().as_uuid().to_string()), "mochu", year - 1, false).await;
    insert_receipt(&pool, Some(&got_other.id().as_uuid().to_string()), "other", year, false).await;
    insert_receipt(&pool, Some(&deleted_nenga.id().as_uuid().to_string()), "nenga", year, true).await;
    insert_receipt(&pool, None, "nenga", year, false).await;

    let rows = repo.list_nenga_target_review(year, 2001).await.unwrap();
    let names: Vec<_> = rows.iter().map(|e| e.primary_name().last().to_string()).collect();
    assert_eq!(names, vec!["伊藤", "山田", "渡辺"]);
  }

  #[tokio::test]
  async fn disable_nenga_send_targets_updates_only_active_true_rows() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool);
    let active = named_entry("山田", None, true);
    let already = named_entry("佐藤", None, false);
    let archived = named_entry("高橋", None, true);
    repo.create(&active).await.unwrap();
    repo.create(&already).await.unwrap();
    repo.create(&archived).await.unwrap();
    repo.archive(archived.id()).await.unwrap();

    let updated = repo
      .disable_nenga_send_targets(
        &[
          active.id().as_uuid().to_string(),
          already.id().as_uuid().to_string(),
          archived.id().as_uuid().to_string(),
          uuid::Uuid::new_v4().to_string(),
        ],
        "2026-10-17T00:00:00+00:00",
      )
      .await
      .unwrap();
    assert_eq!(updated, 1);

    let found = repo.find_by_id(active.id()).await.unwrap().unwrap();
    assert!(!found.nenga_send_target());
    assert_eq!(found.updated_at().to_rfc3339(), "2026-10-17T00:00:00+00:00");
    assert!(repo.find_by_id(archived.id()).await.unwrap().unwrap().nenga_send_target());
  }

  #[tokio::test]
  async fn list_nenga_send_excluded_ids_can_skip_archived() {
    let pool = setup_pool().await;
    let repo = SqlxAddressEntryRepository::new(pool);
    let active = named_entry("山田", None, false);
    let archived = named_entry("佐藤", None, false);
    let target = named_entry("高橋", None, true);
    repo.create(&active).await.unwrap();
    repo.create(&archived).await.unwrap();
    repo.create(&target).await.unwrap();
    repo.archive(archived.id()).await.unwrap();

    let active_only = repo.list_nenga_send_excluded_ids(true).await.unwrap();
    assert_eq!(active_only, vec![active.id().as_uuid()]);
    let all = repo.list_nenga_send_excluded_ids(false).await.unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.contains(&active.id().as_uuid()));
    assert!(all.contains(&archived.id().as_uuid()));
  }
}

