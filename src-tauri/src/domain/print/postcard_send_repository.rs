use crate::domain::print::postcard_send::PostcardSend;
use uuid::Uuid;

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

  /// 指定 print_job_id に既に存在する address_entry_id（未削除）を返す。
  async fn list_address_entry_ids_for_print_job(
    &self,
    print_job_id: Uuid,
  ) -> Result<Vec<Uuid>, PostcardSendRepositoryError>;
}
