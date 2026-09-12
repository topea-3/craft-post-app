use crate::domain::print::postcard_type::PostcardType;
use crate::domain::print::print_layout_preference::PrintLayoutPreference;

#[derive(Debug, thiserror::Error)]
pub enum PrintLayoutPreferenceRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
}

#[async_trait::async_trait]
pub trait PrintLayoutPreferenceRepository {
  async fn list_by_postcard_type(
    &self,
    postcard_type: PostcardType,
  ) -> Result<Vec<PrintLayoutPreference>, PrintLayoutPreferenceRepositoryError>;

  /// 指定種別のオフセットを一括保存（layer_id 単位で upsert）
  async fn save_all(
    &self,
    postcard_type: PostcardType,
    preferences: &[PrintLayoutPreference],
  ) -> Result<(), PrintLayoutPreferenceRepositoryError>;
}
