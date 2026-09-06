use chrono::{DateTime, Local, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::print::postcard_type::PostcardType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostcardSendId(Uuid);

impl PostcardSendId {
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
pub struct PostcardSend {
  id: PostcardSendId,
  print_job_id: Uuid,
  address_entry_id: Uuid,
  sender_entry_id: Uuid,
  sender_snapshot: String,
  address_snapshot: String,
  postcard_type: PostcardType,
  sent_on: NaiveDate,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  deleted_at: Option<DateTime<Utc>>,
}

impl PostcardSend {
  /// OS ローカルタイムゾーンの「今日」（暦日）
  pub fn local_today() -> NaiveDate {
    Local::now().date_naive()
  }

  #[allow(clippy::too_many_arguments)]
  pub fn create_new(
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
    sender_snapshot: String,
    address_snapshot: String,
    postcard_type: PostcardType,
  ) -> Self {
    Self::create_new_as_of(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      Self::local_today(),
    )
  }

  /// 送付日を注入する作成（テスト・タイムゾーン境界の固定用）
  #[allow(clippy::too_many_arguments)]
  pub fn create_new_as_of(
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
    sender_snapshot: String,
    address_snapshot: String,
    postcard_type: PostcardType,
    sent_on: NaiveDate,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: PostcardSendId::new(),
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      sent_on,
      created_at: now,
      updated_at: now,
      deleted_at: None,
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn from_persisted(
    id: PostcardSendId,
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
    sender_snapshot: String,
    address_snapshot: String,
    postcard_type: PostcardType,
    sent_on: NaiveDate,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
  ) -> Self {
    Self {
      id,
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      sent_on,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  pub fn id(&self) -> &PostcardSendId {
    &self.id
  }

  pub fn print_job_id(&self) -> Uuid {
    self.print_job_id
  }

  pub fn address_entry_id(&self) -> Uuid {
    self.address_entry_id
  }

  pub fn sender_entry_id(&self) -> Uuid {
    self.sender_entry_id
  }

  pub fn sender_snapshot(&self) -> &str {
    &self.sender_snapshot
  }

  pub fn address_snapshot(&self) -> &str {
    &self.address_snapshot
  }

  pub fn postcard_type(&self) -> PostcardType {
    self.postcard_type
  }

  pub fn sent_on(&self) -> NaiveDate {
    self.sent_on
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
    self.deleted_at
  }

  pub fn is_deleted(&self) -> bool {
    self.deleted_at.is_some()
  }
}

#[cfg(test)]
mod tests {
  use chrono::NaiveDate;
  use uuid::Uuid;

  use super::*;
  use crate::domain::print::postcard_type::PostcardType;

  #[test]
  fn create_new_as_of_sets_sent_on() {
    let sent_on = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let send = PostcardSend::create_new_as_of(
      Uuid::new_v4(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      "{}".to_string(),
      "{}".to_string(),
      PostcardType::Mochu,
      sent_on,
    );
    assert_eq!(send.sent_on(), sent_on);
    assert_eq!(send.postcard_type(), PostcardType::Mochu);
    assert!(!send.is_deleted());
  }
}
