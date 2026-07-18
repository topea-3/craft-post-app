use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::postcard_receipt::postcard_receipt::{PostcardReceipt, PostcardReceiptId};
use crate::domain::postcard_receipt::postcard_receipt_category::PostcardReceiptCategory;

#[derive(Debug, thiserror::Error)]
pub enum PostcardReceiptRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
  #[error("postcard receipt not found")]
  NotFound,
  /// 紐付け先住所が存在しない、または active 条件を満たさない（並行 archive 等）
  #[error("address entry link rejected")]
  AddressLinkRejected,
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

#[derive(Debug, Clone)]
pub struct PostcardReceiptSearchQuery {
  pub keyword: Option<String>,
  pub year: Option<i32>,
  pub category: Option<PostcardReceiptCategory>,
  pub address_entry_id: Option<Uuid>,
  pub include_deleted: bool,
  pub pagination: Pagination,
  pub sort_order: SortOrder,
}

#[derive(Debug, Clone)]
pub struct PostcardReceiptAddressContext {
  pub display_name: String,
  pub address_line: String,
  pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct PostcardReceiptWithContext {
  pub receipt: PostcardReceipt,
  pub address: Option<PostcardReceiptAddressContext>,
}

/// DB 行用の素の構造体。
#[derive(Debug, Clone)]
pub struct DbPostcardReceiptRow {
  pub id: String,
  pub address_entry_id: Option<String>,
  pub sender_display_name: Option<String>,
  pub received_at: String,
  pub category: String,
  pub memo: Option<String>,
  pub deleted_at: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct DbPostcardReceiptSearchRow {
  pub receipt: DbPostcardReceiptRow,
}

pub fn map_db_row_to_receipt(row: DbPostcardReceiptRow) -> Result<PostcardReceipt, PostcardReceiptRepositoryError> {
  let id = Uuid::parse_str(&row.id)
    .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?;
  let address_entry_id = match row.address_entry_id {
    Some(s) => Some(
      Uuid::parse_str(&s)
        .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?,
    ),
    None => None,
  };
  let received_at = NaiveDate::parse_from_str(&row.received_at, "%Y-%m-%d")
    .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?;
  let category = PostcardReceiptCategory::parse(&row.category)
    .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?;
  let memo = match row.memo {
    Some(text) if !text.is_empty() => Some(
      crate::domain::address::memo::Memo::new(text)
        .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?,
    ),
    _ => None,
  };
  let deleted_at = parse_optional_datetime(row.deleted_at)?;
  let created_at = DateTime::parse_from_rfc3339(&row.created_at)
    .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?
    .with_timezone(&Utc);
  let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
    .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?
    .with_timezone(&Utc);

  PostcardReceipt::from_persisted(
    PostcardReceiptId::from_uuid(id),
    address_entry_id,
    row.sender_display_name,
    received_at,
    category,
    memo,
    deleted_at,
    created_at,
    updated_at,
  )
  .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))
}

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>, PostcardReceiptRepositoryError> {
  match value {
    Some(s) if !s.is_empty() => Ok(Some(
      DateTime::parse_from_rfc3339(&s)
        .map_err(|e| PostcardReceiptRepositoryError::InvalidPersistedData(e.to_string()))?
        .with_timezone(&Utc),
    )),
    _ => Ok(None),
  }
}

#[async_trait::async_trait]
pub trait PostcardReceiptRepository {
  async fn create(
    &self,
    receipt: &PostcardReceipt,
    allow_archived_address_id: Option<Uuid>,
  ) -> Result<(), PostcardReceiptRepositoryError>;

  async fn update(
    &self,
    receipt: &PostcardReceipt,
    allow_archived_address_id: Option<Uuid>,
  ) -> Result<(), PostcardReceiptRepositoryError>;

  async fn find_by_id(
    &self,
    id: &PostcardReceiptId,
  ) -> Result<Option<PostcardReceiptWithContext>, PostcardReceiptRepositoryError>;

  async fn search(
    &self,
    query: PostcardReceiptSearchQuery,
  ) -> Result<(Vec<PostcardReceiptWithContext>, i64), PostcardReceiptRepositoryError>;

  async fn delete(&self, id: &PostcardReceiptId) -> Result<(), PostcardReceiptRepositoryError>;
}
