use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::address::address::{Address, AddressError};
use crate::domain::address::person_name::{PersonName, PersonNameError};
use crate::domain::address::postal_code::{PostalCode, PostalCodeError};
use crate::domain::sender::phone_number::{PhoneNumber, PhoneNumberError};
use crate::domain::sender::sender_label::{SenderLabel, SenderLabelError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderEntryId(Uuid);

impl SenderEntryId {
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

#[derive(Debug, thiserror::Error)]
pub enum SenderEntryError {
  #[error("invalid label: {0}")]
  InvalidLabel(#[from] SenderLabelError),
  #[error("invalid primary name: {0}")]
  InvalidPrimaryName(#[from] PersonNameError),
  #[error("invalid co recipient name: {0}")]
  InvalidCoRecipientName(PersonNameError),
  #[error("invalid postal code: {0}")]
  InvalidPostalCode(#[from] PostalCodeError),
  #[error("invalid address: {0}")]
  InvalidAddress(#[from] AddressError),
  #[error("invalid phone number: {0}")]
  InvalidPhoneNumber(#[from] PhoneNumberError),
  #[error("too many co recipients (max {max}, got {actual})")]
  TooManyCoRecipients { max: usize, actual: usize },
}

#[derive(Debug, Clone)]
pub struct SenderEntry {
  id: SenderEntryId,
  label: SenderLabel,
  primary_name: PersonName,
  co_recipients: Vec<PersonName>,
  postal_code: PostalCode,
  address: Address,
  phone_number: Option<PhoneNumber>,
  archived: bool,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl SenderEntry {
  pub const MAX_CO_RECIPIENTS: usize = 4;

  #[allow(clippy::too_many_arguments)]
  pub fn create_new(
    label: SenderLabel,
    primary_name: PersonName,
    co_recipients: Vec<PersonName>,
    postal_code: PostalCode,
    address: Address,
    phone_number: Option<PhoneNumber>,
  ) -> Result<Self, SenderEntryError> {
    Self::validate_co_recipients_len(&co_recipients)?;
    let now = Utc::now();
    Ok(Self {
      id: SenderEntryId::new(),
      label,
      primary_name,
      co_recipients,
      postal_code,
      address,
      phone_number,
      archived: false,
      created_at: now,
      updated_at: now,
    })
  }

  #[allow(clippy::too_many_arguments)]
  pub fn from_persisted(
    id: SenderEntryId,
    label: SenderLabel,
    primary_name: PersonName,
    co_recipients: Vec<PersonName>,
    postal_code: PostalCode,
    address: Address,
    phone_number: Option<PhoneNumber>,
    archived: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> Result<Self, SenderEntryError> {
    Self::validate_co_recipients_len(&co_recipients)?;
    Ok(Self {
      id,
      label,
      primary_name,
      co_recipients,
      postal_code,
      address,
      phone_number,
      archived,
      created_at,
      updated_at,
    })
  }

  fn validate_co_recipients_len(co_recipients: &[PersonName]) -> Result<(), SenderEntryError> {
    if co_recipients.len() > Self::MAX_CO_RECIPIENTS {
      return Err(SenderEntryError::TooManyCoRecipients {
        max: Self::MAX_CO_RECIPIENTS,
        actual: co_recipients.len(),
      });
    }
    Ok(())
  }

  pub fn id(&self) -> &SenderEntryId {
    &self.id
  }

  pub fn label(&self) -> &SenderLabel {
    &self.label
  }

  pub fn primary_name(&self) -> &PersonName {
    &self.primary_name
  }

  pub fn co_recipients(&self) -> &[PersonName] {
    &self.co_recipients
  }

  pub fn postal_code(&self) -> &PostalCode {
    &self.postal_code
  }

  pub fn address(&self) -> &Address {
    &self.address
  }

  pub fn phone_number(&self) -> Option<&PhoneNumber> {
    self.phone_number.as_ref()
  }

  pub fn archived(&self) -> bool {
    self.archived
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  pub fn display_full_name(&self) -> String {
    PersonName::join_recipients(&self.primary_name, &self.co_recipients)
  }

  pub fn archive(&mut self) {
    self.archived = true;
    self.touch();
  }

  fn touch(&mut self) {
    self.updated_at = Utc::now();
  }
}

