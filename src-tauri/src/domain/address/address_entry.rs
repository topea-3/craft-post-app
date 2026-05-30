use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::address::address::{Address, AddressError};
use crate::domain::address::honorific::Honorific;
use crate::domain::address::memo::{Memo, MemoError};
use crate::domain::address::person_name::{PersonName, PersonNameError};
use crate::domain::address::postal_code::{PostalCode, PostalCodeError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressEntryId(Uuid);

impl AddressEntryId {
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
pub enum AddressEntryError {
  #[error("invalid primary name: {0}")]
  InvalidPrimaryName(#[from] PersonNameError),
  #[error("invalid co recipient name: {0}")]
  InvalidCoRecipientName(PersonNameError),
  #[error("invalid postal code: {0}")]
  InvalidPostalCode(#[from] PostalCodeError),
  #[error("invalid address: {0}")]
  InvalidAddress(#[from] AddressError),
  #[error("invalid memo: {0}")]
  InvalidMemo(#[from] MemoError),
}

#[derive(Debug, Clone)]
pub struct AddressEntry {
  id: AddressEntryId,
  primary_name: PersonName,
  co_recipients: Vec<PersonName>,
  honorific: Honorific,
  postal_code: PostalCode,
  address: Address,
  memo: Option<Memo>,
  archived_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl AddressEntryError {
  /// 補助: 連名者の氏名エラーを `AddressEntryError` に変換する。
  pub fn from_co_recipient(err: PersonNameError) -> Self {
    Self::InvalidCoRecipientName(err)
  }
}

impl AddressEntry {
  #[allow(clippy::too_many_arguments)]
  pub fn create_new(
    primary_name: PersonName,
    co_recipients: Vec<PersonName>,
    honorific: Honorific,
    postal_code: PostalCode,
    address: Address,
    memo: Option<Memo>,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: AddressEntryId::new(),
      primary_name,
      co_recipients,
      honorific,
      postal_code,
      address,
      memo,
      archived_at: None,
      created_at: now,
      updated_at: now,
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn from_persisted(
    id: AddressEntryId,
    primary_name: PersonName,
    co_recipients: Vec<PersonName>,
    honorific: Honorific,
    postal_code: PostalCode,
    address: Address,
    memo: Option<Memo>,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id,
      primary_name,
      co_recipients,
      honorific,
      postal_code,
      address,
      memo,
      archived_at,
      created_at,
      updated_at,
    }
  }

  pub fn id(&self) -> &AddressEntryId {
    &self.id
  }

  pub fn primary_name(&self) -> &PersonName {
    &self.primary_name
  }

  pub fn co_recipients(&self) -> &[PersonName] {
    &self.co_recipients
  }

  pub fn honorific(&self) -> Honorific {
    self.honorific
  }

  pub fn postal_code(&self) -> &PostalCode {
    &self.postal_code
  }

  pub fn address(&self) -> &Address {
    &self.address
  }

  pub fn memo(&self) -> Option<&Memo> {
    self.memo.as_ref()
  }

  pub fn archived(&self) -> bool {
    self.archived_at.is_some()
  }

  pub fn archived_at(&self) -> Option<DateTime<Utc>> {
    self.archived_at
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  /// 一覧用の「氏名＋敬称」表示。
  pub fn display_full_recipient(&self) -> String {
    crate::domain::address::person_name::PersonName::join_recipients_with_honorific(
      &self.primary_name,
      &self.co_recipients,
      &self.honorific,
    )
  }

  /// 論理削除（アーカイブ）日時を記録する（`updated_at` は変えない）。
  pub fn archive(&mut self) {
    self.archived_at = Some(Utc::now());
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domain::address::address::Address;
  use crate::domain::address::honorific::Honorific;
  use crate::domain::address::memo::Memo;
  use crate::domain::address::person_name::PersonName;
  use crate::domain::address::postal_code::PostalCode;

  fn sample_entry() -> AddressEntry {
    let primary = PersonName::new(
      "山田".into(),
      "太郎".into(),
      Some("ヤマダ".into()),
      Some("タロウ".into()),
    )
    .unwrap();
    let co = PersonName::new(
      "山田".into(),
      "花子".into(),
      Some("ヤマダ".into()),
      Some("ハナコ".into()),
    )
    .unwrap();
    let postal = PostalCode::new("1234567").unwrap();
    let addr = Address::new(
      "東京都".into(),
      "渋谷区".into(),
      "神南 1-1-1".into(),
      Some("○○ビル 3F".into()),
    )
    .unwrap();
    let memo = Some(Memo::new("メモ").unwrap());

    AddressEntry::create_new(
      primary,
      vec![co],
      Honorific::Sama,
      postal,
      addr,
      memo,
    )
  }

  #[test]
  fn display_full_recipient_matches_spec() {
    let entry = sample_entry();
    assert_eq!(entry.display_full_recipient(), "山田 太郎・花子 様");
  }

  #[test]
  fn archive_sets_archived_at_without_changing_updated_at() {
    let mut entry = sample_entry();
    let before = entry.updated_at();
    assert!(!entry.archived());

    entry.archive();

    assert!(entry.archived());
    assert_eq!(entry.updated_at(), before);
    assert!(entry.archived_at().is_some());
  }
}

