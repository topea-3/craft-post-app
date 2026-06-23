use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::address::memo::{Memo, MemoError};
use crate::domain::postcard_receipt::postcard_receipt_category::{
  PostcardReceiptCategory, PostcardReceiptCategoryError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostcardReceiptId(Uuid);

impl PostcardReceiptId {
  pub fn new() -> Self {
    Self(Uuid::new_v4())
  }

  pub fn from_uuid(uuid: Uuid) -> Self {
    Self(uuid)
  }

  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostcardReceiptError {
  #[error("received date must not be in the future")]
  FutureReceivedDate,
  #[error("sender display name is required when address entry is not linked")]
  SenderDisplayNameRequired,
  #[error("invalid category: {0}")]
  InvalidCategory(#[from] PostcardReceiptCategoryError),
  #[error("invalid memo: {0}")]
  InvalidMemo(#[from] MemoError),
}

#[derive(Debug, Clone)]
pub struct PostcardReceipt {
  id: PostcardReceiptId,
  address_entry_id: Option<Uuid>,
  sender_display_name: Option<String>,
  received_at: NaiveDate,
  category: PostcardReceiptCategory,
  memo: Option<Memo>,
  deleted_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl PostcardReceipt {
  pub fn create_new(
    address_entry_id: Option<Uuid>,
    sender_display_name: Option<String>,
    received_at: NaiveDate,
    category: PostcardReceiptCategory,
    memo: Option<Memo>,
  ) -> Result<Self, PostcardReceiptError> {
    Self::validate_business_rules(address_entry_id, &sender_display_name, received_at)?;
    let now = Utc::now();
    Ok(Self {
      id: PostcardReceiptId::new(),
      address_entry_id,
      sender_display_name: Self::normalize_sender_display_name(sender_display_name),
      received_at,
      category,
      memo,
      deleted_at: None,
      created_at: now,
      updated_at: now,
    })
  }

  pub fn from_persisted(
    id: PostcardReceiptId,
    address_entry_id: Option<Uuid>,
    sender_display_name: Option<String>,
    received_at: NaiveDate,
    category: PostcardReceiptCategory,
    memo: Option<Memo>,
    deleted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> Result<Self, PostcardReceiptError> {
    Self::validate_business_rules(address_entry_id, &sender_display_name, received_at)?;
    Ok(Self {
      id,
      address_entry_id,
      sender_display_name: Self::normalize_sender_display_name(sender_display_name),
      received_at,
      category,
      memo,
      deleted_at,
      created_at,
      updated_at,
    })
  }

  fn validate_business_rules(
    address_entry_id: Option<Uuid>,
    sender_display_name: &Option<String>,
    received_at: NaiveDate,
  ) -> Result<(), PostcardReceiptError> {
    let today = Utc::now().date_naive();
    if received_at > today {
      return Err(PostcardReceiptError::FutureReceivedDate);
    }
    if address_entry_id.is_none() {
      let name = sender_display_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
      if name.is_empty() {
        return Err(PostcardReceiptError::SenderDisplayNameRequired);
      }
    }
    Ok(())
  }

  fn normalize_sender_display_name(value: Option<String>) -> Option<String> {
    value.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
  }

  pub fn id(&self) -> &PostcardReceiptId {
    &self.id
  }

  pub fn address_entry_id(&self) -> Option<Uuid> {
    self.address_entry_id
  }

  pub fn sender_display_name(&self) -> Option<&str> {
    self.sender_display_name.as_deref()
  }

  pub fn received_at(&self) -> NaiveDate {
    self.received_at
  }

  pub fn category(&self) -> PostcardReceiptCategory {
    self.category
  }

  pub fn memo(&self) -> Option<&Memo> {
    self.memo.as_ref()
  }

  pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
    self.deleted_at
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  pub fn is_deleted(&self) -> bool {
    self.deleted_at.is_some()
  }

  pub fn touch_updated_at(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn mark_deleted(&mut self) {
    let now = Utc::now();
    self.deleted_at = Some(now);
    self.updated_at = now;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn today() -> NaiveDate {
    Utc::now().date_naive()
  }

  #[test]
  fn create_new_requires_sender_display_name_when_unlinked() {
    let err = PostcardReceipt::create_new(
      None,
      None,
      today(),
      PostcardReceiptCategory::Nenga,
      None,
    )
    .expect_err("should require display name");
    assert_eq!(err, PostcardReceiptError::SenderDisplayNameRequired);
  }

  #[test]
  fn create_new_rejects_future_received_date() {
    let future = today() + chrono::Duration::days(1);
    let err = PostcardReceipt::create_new(
      None,
      Some("田中家".to_string()),
      future,
      PostcardReceiptCategory::Nenga,
      None,
    )
    .expect_err("should reject future date");
    assert_eq!(err, PostcardReceiptError::FutureReceivedDate);
  }

  #[test]
  fn create_new_allows_linked_without_display_name() {
    let receipt = PostcardReceipt::create_new(
      Some(Uuid::new_v4()),
      None,
      today(),
      PostcardReceiptCategory::Nenga,
      None,
    )
    .expect("linked receipt should succeed");
    assert!(receipt.sender_display_name().is_none());
  }
}
