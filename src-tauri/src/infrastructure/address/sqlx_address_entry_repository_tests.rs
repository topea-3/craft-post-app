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

    AddressEntry::create_new(primary, vec![co], Honorific::Sama, postal, addr, memo)
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
}

