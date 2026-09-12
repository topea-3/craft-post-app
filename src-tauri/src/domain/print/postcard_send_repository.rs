use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::address::memo::Memo;
use crate::domain::print::postcard_send::{PostcardSend, PostcardSendId};
use crate::domain::print::postcard_send_source::PostcardSendSource;
use crate::domain::print::postcard_type::PostcardType;

#[derive(Debug, thiserror::Error)]
pub enum PostcardSendRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
  #[error("postcard send not found")]
  NotFound,
  /// `(print_job_id, address_entry_id)` UNIQUE 衝突（印刷コマンド層で冪等成功にマップ）
  #[error("postcard send already exists for print job and address")]
  Conflict,
  /// 他操作による更新済み（楽観ロック不一致）
  #[error("postcard send was updated concurrently")]
  OptimisticLockConflict,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub limit: i64,
  pub offset: i64,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
  Asc,
  Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendStatusFilter {
  Sent,
  Unsent,
}

impl SendStatusFilter {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Sent => "sent",
      Self::Unsent => "unsent",
    }
  }

  pub fn parse(value: &str) -> Option<Self> {
    match value {
      "sent" => Some(Self::Sent),
      "unsent" => Some(Self::Unsent),
      _ => None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PostcardSendSearchQuery {
  pub keyword: Option<String>,
  pub year: Option<i32>,
  pub postcard_type: Option<PostcardType>,
  pub address_entry_id: Option<Uuid>,
  pub source: Option<PostcardSendSource>,
  pub include_deleted: bool,
  pub pagination: Pagination,
  pub sort_order: SortOrder,
}

#[derive(Debug, Clone)]
pub struct SendStatusQuery {
  pub year: i32,
  pub postcard_type: Option<PostcardType>,
  pub status: SendStatusFilter,
  pub receipt_year: Option<i32>,
  pub keyword: Option<String>,
  pub pagination: Pagination,
}

#[derive(Debug, Clone)]
pub struct PostcardSendAddressContext {
  pub display_name: String,
  pub address_line: String,
  pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct PostcardSendSenderContext {
  pub label: String,
  pub display_name: String,
  pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct PostcardSendWithContext {
  pub send: PostcardSend,
  pub address: Option<PostcardSendAddressContext>,
  pub sender: Option<PostcardSendSenderContext>,
}

#[derive(Debug, Clone)]
pub struct SendStatusItem {
  pub address_entry_id: Uuid,
  pub display_name: String,
  pub address_summary: String,
  pub last_sent_on: Option<NaiveDate>,
  pub send_count: i64,
}

/// DB 行用の素の構造体。
#[derive(Debug, Clone)]
pub struct DbPostcardSendRow {
  pub id: String,
  pub print_job_id: String,
  pub address_entry_id: String,
  pub sender_entry_id: String,
  pub sender_snapshot: String,
  pub address_snapshot: String,
  pub postcard_type: String,
  pub sent_on: String,
  pub source: String,
  pub memo: Option<String>,
  pub created_at: String,
  pub updated_at: String,
  pub deleted_at: Option<String>,
}

pub fn map_db_row_to_send(row: DbPostcardSendRow) -> Result<PostcardSend, PostcardSendRepositoryError> {
  let id = Uuid::parse_str(&row.id)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let print_job_id = Uuid::parse_str(&row.print_job_id)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let address_entry_id = Uuid::parse_str(&row.address_entry_id)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let sender_entry_id = Uuid::parse_str(&row.sender_entry_id)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let postcard_type = PostcardType::from_str(&row.postcard_type)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let sent_on = NaiveDate::parse_from_str(&row.sent_on, "%Y-%m-%d")
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let source = PostcardSendSource::from_str(&row.source)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
  let memo = match row.memo {
    Some(text) if !text.is_empty() => Some(
      Memo::new(text)
        .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?,
    ),
    _ => None,
  };
  let deleted_at = parse_optional_datetime(row.deleted_at)?;
  let created_at = DateTime::parse_from_rfc3339(&row.created_at)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?
    .with_timezone(&Utc);
  let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
    .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?
    .with_timezone(&Utc);

  Ok(PostcardSend::from_persisted(
    PostcardSendId::from_uuid(id),
    print_job_id,
    address_entry_id,
    sender_entry_id,
    row.sender_snapshot,
    row.address_snapshot,
    postcard_type,
    sent_on,
    source,
    memo,
    created_at,
    updated_at,
    deleted_at,
  ))
}

fn parse_optional_datetime(
  value: Option<String>,
) -> Result<Option<DateTime<Utc>>, PostcardSendRepositoryError> {
  match value {
    Some(s) if !s.is_empty() => Ok(Some(
      DateTime::parse_from_rfc3339(&s)
        .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?
        .with_timezone(&Utc),
    )),
    _ => Ok(None),
  }
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

  async fn find_by_id(
    &self,
    id: &PostcardSendId,
  ) -> Result<Option<PostcardSendWithContext>, PostcardSendRepositoryError>;

  async fn search(
    &self,
    query: PostcardSendSearchQuery,
  ) -> Result<(Vec<PostcardSendWithContext>, i64), PostcardSendRepositoryError>;

  async fn update(
    &self,
    send: &PostcardSend,
    expected_updated_at: &str,
  ) -> Result<(), PostcardSendRepositoryError>;

  async fn delete(&self, id: &PostcardSendId) -> Result<(), PostcardSendRepositoryError>;

  /// 有効な送付履歴に存在する送付年（降順）
  async fn list_sent_years(&self) -> Result<Vec<i32>, PostcardSendRepositoryError>;

  async fn search_send_status(
    &self,
    query: SendStatusQuery,
  ) -> Result<(Vec<SendStatusItem>, i64), PostcardSendRepositoryError>;
}
