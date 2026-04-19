use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::address::address::Address;
use crate::domain::address::address_entry::AddressEntry;
use crate::domain::address::person_name::PersonName;
use crate::domain::address::postal_code::PostalCode;
use crate::domain::sender::phone_number::PhoneNumber;
use crate::domain::sender::sender_entry::{SenderEntry, SenderEntryId};
use crate::domain::sender::sender_label::SenderLabel;

#[derive(Debug, thiserror::Error)]
pub enum SenderRepositoryError {
  #[error("database error: {0}")]
  Db(#[from] sqlx::Error),
  #[error("invalid persisted data: {0}")]
  InvalidPersistedData(String),
  #[error("sender entry not found")]
  NotFound,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub limit: i64,
  pub offset: i64,
}

#[async_trait::async_trait]
pub trait SenderEntryRepository {
  async fn create(&self, entry: &SenderEntry) -> Result<(), SenderRepositoryError>;
  async fn update(&self, entry: &SenderEntry) -> Result<(), SenderRepositoryError>;
  async fn exists_active_label(
    &self,
    label: &str,
    exclude_id: Option<&SenderEntryId>,
  ) -> Result<bool, SenderRepositoryError>;
  async fn find_by_id(&self, id: &SenderEntryId) -> Result<Option<SenderEntry>, SenderRepositoryError>;

  /// 差出人に紐づく宛名を、リンクテーブルの並び順で取得する。
  async fn list_linked_address_entries(
    &self,
    sender_entry_id: &SenderEntryId,
  ) -> Result<Vec<AddressEntry>, SenderRepositoryError>;

  async fn list_active(
    &self,
    pagination: Pagination,
  ) -> Result<Vec<SenderEntry>, SenderRepositoryError>;
  async fn archive(&self, id: &SenderEntryId) -> Result<(), SenderRepositoryError>;
  async fn find_sender_id_by_address_entry_id(
    &self,
    address_entry_id: Uuid,
  ) -> Result<Option<SenderEntryId>, SenderRepositoryError>;
  async fn replace_links_for_sender(
    &self,
    sender_entry_id: &SenderEntryId,
    address_entry_ids: &[Uuid],
  ) -> Result<(), SenderRepositoryError>;

  /// 宛名 1 件に対する差出人紐づけを設定する（Some で紐づけ、None で解除）。
  /// 宛名側は「高々 1 件」を前提に、既存リンクは削除して差し替える。
  async fn set_sender_for_address(
    &self,
    address_entry_id: Uuid,
    sender_entry_id: Option<&SenderEntryId>,
  ) -> Result<(), SenderRepositoryError>;
}

#[derive(Debug)]
pub struct DbSenderEntryRow {
  pub id: String,
  pub label: String,
  pub primary_last: String,
  pub primary_first: String,
  pub primary_kana_last: Option<String>,
  pub primary_kana_first: Option<String>,
  pub postal_code: String,
  pub prefecture: String,
  pub city: String,
  pub street: String,
  pub building: Option<String>,
  pub phone_number: Option<String>,
  pub archived_at: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug)]
pub struct DbSenderCoRecipientRow {
  pub last: String,
  pub first: String,
  pub kana_last: Option<String>,
  pub kana_first: Option<String>,
}

impl DbSenderEntryRow {
  pub fn into_domain(
    self,
    co_recipients: Vec<DbSenderCoRecipientRow>,
  ) -> Result<SenderEntry, SenderRepositoryError> {
    let id = SenderEntryId::from_uuid(
      Uuid::parse_str(&self.id)
        .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?,
    );
    let label = SenderLabel::new(self.label)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
    let primary_name = PersonName::new(
      self.primary_last,
      self.primary_first,
      self.primary_kana_last,
      self.primary_kana_first,
    )
    .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
    let mut co_names = Vec::with_capacity(co_recipients.len());
    for row in co_recipients {
      let name = PersonName::new(row.last, row.first, row.kana_last, row.kana_first)
        .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
      co_names.push(name);
    }
    let postal_code = PostalCode::new(self.postal_code)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
    let address = Address::new(self.prefecture, self.city, self.street, self.building)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
    let phone_number = match self.phone_number {
      Some(text) => Some(
        PhoneNumber::new(text)
          .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?,
      ),
      None => None,
    };
    let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&self.created_at)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?
      .with_timezone(&Utc);
    let updated_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&self.updated_at)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?
      .with_timezone(&Utc);
    let archived_at = match self.archived_at.as_deref() {
      None | Some("") => None,
      Some(s) => Some(
        DateTime::parse_from_rfc3339(s)
          .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?
          .with_timezone(&Utc),
      ),
    };
    SenderEntry::from_persisted(
      id,
      label,
      primary_name,
      co_names,
      postal_code,
      address,
      phone_number,
      archived_at,
      created_at,
      updated_at,
    )
    .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))
  }
}

