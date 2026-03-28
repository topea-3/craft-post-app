#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;
  use uuid::Uuid;

  use crate::domain::address::address::Address;
  use crate::domain::address::person_name::PersonName;
  use crate::domain::address::postal_code::PostalCode;
  use crate::domain::sender::phone_number::PhoneNumber;
  use crate::domain::sender::sender_entry::SenderEntry;
  use crate::domain::sender::sender_entry_repository::{Pagination, SenderEntryRepository};
  use crate::domain::sender::sender_label::SenderLabel;
  use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

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

  async fn insert_address_entry(pool: &SqlitePool, id: Uuid) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id, primary_last, primary_first, primary_kana_last, primary_kana_first,
          honorific, postal_code, prefecture, city, street, building, memo, archived, created_at, updated_at
        )
        VALUES (?, '佐藤', '一郎', NULL, NULL, '様', '1234567', '東京都', '千代田区', '1-1-1', NULL, NULL, 0, ?, ?)
      "#,
    )
    .bind(id.to_string())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
  }

  fn sample_entry(label: &str) -> SenderEntry {
    let label = SenderLabel::new(label.to_string()).unwrap();
    let primary = PersonName::new("山田".into(), "太郎".into(), None, None).unwrap();
    let co = PersonName::new("山田".into(), "花子".into(), None, None).unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new("東京都".into(), "渋谷区".into(), "神南 1-1-1".into(), None).unwrap();
    let phone = Some(PhoneNumber::new("03-1111-2222".into()).unwrap());
    SenderEntry::create_new(label, primary, vec![co], postal, addr, phone).unwrap()
  }

  #[tokio::test]
  async fn create_and_find_roundtrip() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool);
    let entry = sample_entry("自宅");
    let id = entry.id().clone();
    repo.create(&entry).await.unwrap();
    let found = repo.find_by_id(&id).await.unwrap().expect("sender not found");
    assert_eq!(found.label().value(), "自宅");
    assert_eq!(found.display_full_name(), "山田 太郎・花子");
  }

  #[tokio::test]
  async fn archive_removes_links() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    let entry = sample_entry("会社");
    let sender_id = entry.id().clone();
    repo.create(&entry).await.unwrap();

    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id).await;
    repo
      .replace_links_for_sender(&sender_id, &[address_id])
      .await
      .unwrap();
    assert!(repo
      .find_sender_id_by_address_entry_id(address_id)
      .await
      .unwrap()
      .is_some());

    repo.archive(&sender_id).await.unwrap();
    assert!(repo
      .find_sender_id_by_address_entry_id(address_id)
      .await
      .unwrap()
      .is_none());
  }

  #[tokio::test]
  async fn replace_links_moves_address_to_new_sender() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    let sender_a = sample_entry("A");
    let sender_b = sample_entry("B");
    let sender_a_id = sender_a.id().clone();
    let sender_b_id = sender_b.id().clone();
    repo.create(&sender_a).await.unwrap();
    repo.create(&sender_b).await.unwrap();

    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id).await;
    repo
      .replace_links_for_sender(&sender_a_id, &[address_id])
      .await
      .unwrap();
    repo
      .replace_links_for_sender(&sender_b_id, &[address_id])
      .await
      .unwrap();

    let linked_sender = repo
      .find_sender_id_by_address_entry_id(address_id)
      .await
      .unwrap()
      .expect("link should exist");
    assert_eq!(linked_sender.as_uuid(), sender_b_id.as_uuid());

    let list_a = repo
      .list_active(Pagination { limit: 10, offset: 0 })
      .await
      .unwrap();
    assert_eq!(list_a.len(), 2);
  }

  #[tokio::test]
  async fn exists_active_label_works_with_exclude_id() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool);
    let sender_a = sample_entry("重複ラベル");
    let sender_b = sample_entry("別ラベル");
    let sender_a_id = sender_a.id().clone();
    let sender_b_id = sender_b.id().clone();
    repo.create(&sender_a).await.unwrap();
    repo.create(&sender_b).await.unwrap();

    assert!(repo
      .exists_active_label("重複ラベル", None)
      .await
      .unwrap());
    assert!(!repo
      .exists_active_label("重複ラベル", Some(&sender_a_id))
      .await
      .unwrap());
    assert!(repo
      .exists_active_label("重複ラベル", Some(&sender_b_id))
      .await
      .unwrap());
  }

  #[tokio::test]
  async fn list_linked_address_entries_excludes_archived_addresses() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    let entry = sample_entry("アーカイブ宛名は紐づき一覧から除外");
    let sender_id = entry.id().clone();
    repo.create(&entry).await.unwrap();

    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id).await;
    repo
      .replace_links_for_sender(&sender_id, &[address_id])
      .await
      .unwrap();

    assert_eq!(
      repo
        .list_linked_address_entries(&sender_id)
        .await
        .unwrap()
        .len(),
      1
    );

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE address_entries SET archived = 1, updated_at = ? WHERE id = ?")
      .bind(&now)
      .bind(address_id.to_string())
      .execute(&pool)
      .await
      .unwrap();

    assert!(repo
      .list_linked_address_entries(&sender_id)
      .await
      .unwrap()
      .is_empty());
  }

  #[tokio::test]
  async fn set_sender_for_address_entry_can_set_and_unset() {
    let pool = setup_pool().await;
    let repo = SqlxSenderEntryRepository::new(pool.clone());
    let entry = sample_entry("宛名側セット");
    let sender_id = entry.id().clone();
    repo.create(&entry).await.unwrap();

    let address_id = Uuid::new_v4();
    insert_address_entry(&pool, address_id).await;

    // set
    repo
      .set_sender_for_address(address_id, Some(&sender_id))
      .await
      .unwrap();
    assert_eq!(
      repo.find_sender_id_by_address_entry_id(address_id).await.unwrap(),
      Some(sender_id.clone())
    );

    // unset
    repo.set_sender_for_address(address_id, None).await.unwrap();
    assert_eq!(
      repo.find_sender_id_by_address_entry_id(address_id).await.unwrap(),
      None
    );
  }
}

