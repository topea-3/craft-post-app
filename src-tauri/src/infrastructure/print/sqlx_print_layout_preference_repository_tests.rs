#[cfg(test)]
mod tests {
  use sqlx::SqlitePool;

  use crate::domain::print::postcard_type::PostcardType;
  use crate::domain::print::print_layout_preference::PrintLayoutPreference;
  use crate::domain::print::print_layout_preference_repository::PrintLayoutPreferenceRepository;
  use crate::infrastructure::print::sqlx_print_layout_preference_repository::SqlxPrintLayoutPreferenceRepository;

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

  #[tokio::test]
  async fn save_all_and_list_by_postcard_type_roundtrip() {
    let pool = setup_pool().await;
    let repo = SqlxPrintLayoutPreferenceRepository::new(pool);

    let prefs = vec![
      PrintLayoutPreference::create_new(
        PostcardType::Nenga,
        "recipient.postalCode".to_string(),
        1.0,
        2.0,
      ),
      PrintLayoutPreference::create_new(
        PostcardType::Nenga,
        "sender.address1".to_string(),
        -3.5,
        4.25,
      ),
    ];

    repo
      .save_all(PostcardType::Nenga, &prefs)
      .await
      .expect("save_all");

    let listed = repo
      .list_by_postcard_type(PostcardType::Nenga)
      .await
      .expect("list nenga");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].layer_id(), "recipient.postalCode");
    assert_eq!(listed[0].offset_x_pt(), 1.0);
    assert_eq!(listed[0].offset_y_pt(), 2.0);
    assert_eq!(listed[1].layer_id(), "sender.address1");
    assert_eq!(listed[1].offset_x_pt(), -3.5);
    assert_eq!(listed[1].offset_y_pt(), 4.25);

    let updated = vec![PrintLayoutPreference::create_new(
      PostcardType::Nenga,
      "recipient.postalCode".to_string(),
      10.0,
      20.0,
    )];
    repo
      .save_all(PostcardType::Nenga, &updated)
      .await
      .expect("upsert");

    let listed = repo
      .list_by_postcard_type(PostcardType::Nenga)
      .await
      .expect("list after upsert");
    let postal = listed
      .iter()
      .find(|p| p.layer_id() == "recipient.postalCode")
      .expect("postal");
    assert_eq!(postal.offset_x_pt(), 10.0);
    assert_eq!(postal.offset_y_pt(), 20.0);

    let mochu = repo
      .list_by_postcard_type(PostcardType::Mochu)
      .await
      .expect("list mochu");
    assert!(mochu.is_empty());
  }
}
