use crate::domain::print::postcard_send::PostcardSend;

#[derive(Debug, thiserror::Error)]
pub enum PostcardSendRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
  /// `(print_job_id, address_entry_id)` UNIQUE 衝突（コマンド層で冪等成功にマップ）
  #[error("postcard send already exists for print job and address")]
  Conflict,
}

#[async_trait::async_trait]
pub trait PostcardSendRepository {
  /// 複数件をトランザクションで INSERT。UNIQUE 衝突時は `Conflict`。
  async fn create_batch(
    &self,
    sends: &[PostcardSend],
  ) -> Result<(), PostcardSendRepositoryError>;
}
