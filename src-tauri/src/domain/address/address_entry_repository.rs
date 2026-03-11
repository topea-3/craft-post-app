use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::address::address::Address;
use crate::domain::address::address_entry::{AddressEntry, AddressEntryId};
use crate::domain::address::honorific::Honorific;
use crate::domain::address::memo::Memo;
use crate::domain::address::person_name::PersonName;
use crate::domain::address::postal_code::PostalCode;

#[derive(Debug, thiserror::Error)]
pub enum AddressRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
}

/// 一覧取得時のページング情報。
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub limit: i64,
  pub offset: i64,
}

/// 一覧・検索条件。
#[derive(Debug, Clone)]
pub enum SortKey {
  NameKana,
  UpdatedAt,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
  Asc,
  Desc,
}

#[derive(Debug, Clone)]
pub struct AddressSearchQuery {
  pub keyword: Option<String>,
  pub sort_key: SortKey,
  pub sort_order: SortOrder,
  pub include_archived: bool,
  pub pagination: Option<Pagination>,
}

/// AddressEntry 用リポジトリトレイト。
///
/// インフラ層で sqlx を使って実装する。
#[async_trait::async_trait]
pub trait AddressEntryRepository {
  async fn create(&self, entry: &AddressEntry) -> Result<(), AddressRepositoryError>;

  async fn update(&self, entry: &AddressEntry) -> Result<(), AddressRepositoryError>;

  async fn find_by_id(
    &self,
    id: &AddressEntryId,
  ) -> Result<Option<AddressEntry>, AddressRepositoryError>;

  async fn list_active(
    &self,
    pagination: Pagination,
  ) -> Result<Vec<AddressEntry>, AddressRepositoryError>;

  async fn search(
    &self,
    query: AddressSearchQuery,
  ) -> Result<Vec<AddressEntry>, AddressRepositoryError>;

  async fn archive(&self, id: &AddressEntryId) -> Result<(), AddressRepositoryError>;
}

/// DB 行用の素の構造体。
///
/// 読み込み時の後方互換性を確保するため、ここではバリデーションを行わず、
/// 可能な限りそのまま保持する。
#[derive(Debug)]
pub struct DbAddressEntryRow {
  pub id: String,
  pub primary_last: String,
  pub primary_first: String,
  pub primary_kana_last: Option<String>,
  pub primary_kana_first: Option<String>,
  pub honorific: String,
  pub postal_code: String,
  pub prefecture: String,
  pub city: String,
  pub street: String,
  pub building: Option<String>,
  pub memo: Option<String>,
  pub archived: bool,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug)]
pub struct DbCoRecipientRow {
  pub id: String,
  pub address_entry_id: String,
  pub order_index: i64,
  pub last: String,
  pub first: String,
  pub kana_last: Option<String>,
  pub kana_first: Option<String>,
  pub archived: bool,
  pub created_at: String,
  pub updated_at: String,
}

impl DbAddressEntryRow {
  pub fn into_domain(
    self,
    co_recipients: Vec<DbCoRecipientRow>,
  ) -> Result<AddressEntry, AddressRepositoryError> {
    let id = AddressEntryId::from_uuid(
      Uuid::parse_str(&self.id)
        .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?,
    );

    let primary_name = PersonName::new(
      self.primary_last,
      self.primary_first,
      self.primary_kana_last,
      self.primary_kana_first,
    )
    .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?;

    let mut co_names = Vec::with_capacity(co_recipients.len());
    for row in co_recipients {
      let name = PersonName::new(row.last, row.first, row.kana_last, row.kana_first)
        .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?;
      co_names.push(name);
    }

    let honorific = Honorific::from_str(&self.honorific)
      .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?;

    let postal_code = PostalCode::new(self.postal_code)
      .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?;

    let address = Address::new(self.prefecture, self.city, self.street, self.building)
      .map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?;

    let memo = match self.memo {
      Some(text) => Some(
        Memo::new(text).map_err(|e| AddressRepositoryError::InvalidPersistedData(e.to_string()))?,
      ),
      None => None,
    };

    let created_at: DateTime<Utc> = match DateTime::parse_from_rfc3339(&self.created_at) {
      Ok(dt) => dt.with_timezone(&Utc),
      Err(e) => {
        return Err(AddressRepositoryError::InvalidPersistedData(e.to_string()));
      }
    };
    let updated_at: DateTime<Utc> = match DateTime::parse_from_rfc3339(&self.updated_at) {
      Ok(dt) => dt.with_timezone(&Utc),
      Err(e) => {
        return Err(AddressRepositoryError::InvalidPersistedData(e.to_string()));
      }
    };

    Ok(AddressEntry::from_persisted(
      id,
      primary_name,
      co_names,
      honorific,
      postal_code,
      address,
      memo,
      self.archived,
      created_at,
      updated_at,
    ))
  }
}

