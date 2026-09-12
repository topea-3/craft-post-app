use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::sender::sender_entry::SenderEntryId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderAddressLinkId(Uuid);

impl SenderAddressLinkId {
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

#[derive(Debug, Clone)]
pub struct SenderAddressLink {
  id: SenderAddressLinkId,
  sender_entry_id: SenderEntryId,
  address_entry_id: Uuid,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl SenderAddressLink {
  pub fn create_new(sender_entry_id: SenderEntryId, address_entry_id: Uuid) -> Self {
    let now = Utc::now();
    Self {
      id: SenderAddressLinkId::new(),
      sender_entry_id,
      address_entry_id,
      created_at: now,
      updated_at: now,
    }
  }

  pub fn from_persisted(
    id: SenderAddressLinkId,
    sender_entry_id: SenderEntryId,
    address_entry_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id,
      sender_entry_id,
      address_entry_id,
      created_at,
      updated_at,
    }
  }

  pub fn id(&self) -> &SenderAddressLinkId {
    &self.id
  }

  pub fn sender_entry_id(&self) -> &SenderEntryId {
    &self.sender_entry_id
  }

  pub fn address_entry_id(&self) -> Uuid {
    self.address_entry_id
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }
}

