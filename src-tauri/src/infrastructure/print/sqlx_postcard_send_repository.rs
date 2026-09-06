use sqlx::SqlitePool;

use crate::domain::print::postcard_send::PostcardSend;
use crate::domain::print::postcard_send_repository::{
  PostcardSendRepository, PostcardSendRepositoryError,
};

pub struct SqlxPostcardSendRepository {
  pool: SqlitePool,
}

impl SqlxPostcardSendRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

fn is_job_address_unique_violation(err: &sqlx::Error) -> bool {
  match err {
    sqlx::Error::Database(db_err) => {
      let msg = db_err.message();
      msg.contains("UNIQUE constraint failed")
        && (msg.contains("idx_postcard_sends_job_address")
          || msg.contains("print_job_id")
          || msg.contains("postcard_sends"))
    }
    _ => false,
  }
}

#[async_trait::async_trait]
impl PostcardSendRepository for SqlxPostcardSendRepository {
  async fn create_batch(
    &self,
    sends: &[PostcardSend],
  ) -> Result<(), PostcardSendRepositoryError> {
    if sends.is_empty() {
      return Ok(());
    }

    let mut tx = self.pool.begin().await?;

    for send in sends {
      let id = send.id().as_uuid().to_string();
      let print_job_id = send.print_job_id().to_string();
      let address_entry_id = send.address_entry_id().to_string();
      let sender_entry_id = send.sender_entry_id().to_string();
      let sender_snapshot = send.sender_snapshot();
      let address_snapshot = send.address_snapshot();
      let postcard_type = send.postcard_type().as_str();
      let sent_on = send.sent_on().format("%Y-%m-%d").to_string();
      let created_at = send.created_at().to_rfc3339();
      let updated_at = send.updated_at().to_rfc3339();
      let deleted_at = send.deleted_at().map(|t| t.to_rfc3339());

      let result = sqlx::query(
        r#"
          INSERT INTO postcard_sends (
            id, print_job_id, address_entry_id, sender_entry_id,
            sender_snapshot, address_snapshot, postcard_type,
            sent_on, created_at, updated_at, deleted_at
          )
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
      )
      .bind(&id)
      .bind(&print_job_id)
      .bind(&address_entry_id)
      .bind(&sender_entry_id)
      .bind(sender_snapshot)
      .bind(address_snapshot)
      .bind(postcard_type)
      .bind(&sent_on)
      .bind(&created_at)
      .bind(&updated_at)
      .bind(deleted_at)
      .execute(&mut *tx)
      .await;

      match result {
        Ok(_) => {}
        Err(e) if is_job_address_unique_violation(&e) => {
          // トランザクションをロールバックして Conflict を返す（コマンド層で冪等成功）
          drop(tx);
          return Err(PostcardSendRepositoryError::Conflict);
        }
        Err(e) => return Err(PostcardSendRepositoryError::Db(e)),
      }
    }

    tx.commit().await?;
    Ok(())
  }
}
