use chrono::{DateTime, Local, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::address::memo::{Memo, MemoError};
use crate::domain::print::postcard_send_source::PostcardSendSource;
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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostcardSendError {
  #[error("sent date must not be in the future")]
  FutureSentDate,
  #[error("invalid memo: {0}")]
  InvalidMemo(#[from] MemoError),
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
  source: PostcardSendSource,
  memo: Option<Memo>,
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
    source: PostcardSendSource,
    memo: Option<Memo>,
  ) -> Result<Self, PostcardSendError> {
    let today = Self::local_today();
    Self::create_new_as_of(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      today,
      source,
      memo,
      today,
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
    source: PostcardSendSource,
    memo: Option<Memo>,
    today: NaiveDate,
  ) -> Result<Self, PostcardSendError> {
    Self::validate_sent_on_not_future(sent_on, today)?;
    let now = Utc::now();
    Ok(Self {
      id: PostcardSendId::new(),
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      sent_on,
      source,
      memo,
      created_at: now,
      updated_at: now,
      deleted_at: None,
    })
  }

  /// DB 再構成。時刻依存の未来日検証は行わない。
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
    source: PostcardSendSource,
    memo: Option<Memo>,
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
      source,
      memo,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  /// update 入力境界用: 送付日を変更する場合のみ未来日検証する。
  /// 変更可能なのは `sent_on` / `postcard_type` / `memo` のみ。
  #[allow(clippy::too_many_arguments)]
  pub fn from_persisted_for_update(
    id: PostcardSendId,
    print_job_id: Uuid,
    address_entry_id: Uuid,
    sender_entry_id: Uuid,
    sender_snapshot: String,
    address_snapshot: String,
    postcard_type: PostcardType,
    sent_on: NaiveDate,
    source: PostcardSendSource,
    memo: Option<Memo>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    previous_sent_on: NaiveDate,
    today: NaiveDate,
  ) -> Result<Self, PostcardSendError> {
    if sent_on != previous_sent_on {
      Self::validate_sent_on_not_future(sent_on, today)?;
    }
    Ok(Self::from_persisted(
      id,
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
      sent_on,
      source,
      memo,
      created_at,
      updated_at,
      deleted_at,
    ))
  }

  pub fn validate_sent_on_not_future(
    sent_on: NaiveDate,
    today: NaiveDate,
  ) -> Result<(), PostcardSendError> {
    if sent_on > today {
      return Err(PostcardSendError::FutureSentDate);
    }
    Ok(())
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

  pub fn source(&self) -> PostcardSendSource {
    self.source
  }

  pub fn memo(&self) -> Option<&Memo> {
    self.memo.as_ref()
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
  use chrono::NaiveDate;
  use uuid::Uuid;

  use super::*;
  use crate::domain::print::postcard_send_source::PostcardSendSource;
  use crate::domain::print::postcard_type::PostcardType;

  fn fixed_today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
  }

  fn base_create(
    sent_on: NaiveDate,
    source: PostcardSendSource,
    memo: Option<Memo>,
  ) -> Result<PostcardSend, PostcardSendError> {
    PostcardSend::create_new_as_of(
      Uuid::new_v4(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      "{}".to_string(),
      "{}".to_string(),
      PostcardType::Mochu,
      sent_on,
      source,
      memo,
      fixed_today(),
    )
  }

  #[test]
  fn create_new_as_of_sets_fields() {
    let sent_on = fixed_today();
    let send = base_create(sent_on, PostcardSendSource::Print, None).expect("create");
    assert_eq!(send.sent_on(), sent_on);
    assert_eq!(send.postcard_type(), PostcardType::Mochu);
    assert_eq!(send.source(), PostcardSendSource::Print);
    assert!(send.memo().is_none());
    assert!(!send.is_deleted());
  }

  #[test]
  fn create_rejects_future_sent_on() {
    let tomorrow = fixed_today() + chrono::Duration::days(1);
    let err = base_create(tomorrow, PostcardSendSource::Manual, None)
      .expect_err("future must be rejected");
    assert_eq!(err, PostcardSendError::FutureSentDate);
  }

  #[test]
  fn create_allows_today() {
    let send = base_create(fixed_today(), PostcardSendSource::Manual, None).expect("today ok");
    assert_eq!(send.sent_on(), fixed_today());
  }

  #[test]
  fn create_rejects_memo_over_1000_codepoints() {
    let long = "あ".repeat(1001);
    let memo_err = Memo::new(long).expect_err("memo too long");
    assert!(matches!(memo_err, MemoError::TooLong { max: 1000 }));
  }

  #[test]
  fn create_accepts_memo_at_1000_codepoints() {
    let memo = Memo::new("a".repeat(1000)).expect("1000 ok");
    let send = base_create(fixed_today(), PostcardSendSource::Manual, Some(memo)).expect("create");
    assert_eq!(send.memo().map(|m| m.text().chars().count()), Some(1000));
  }

  #[test]
  fn from_persisted_allows_future_looking_date() {
    let tomorrow = fixed_today() + chrono::Duration::days(1);
    let send = PostcardSend::from_persisted(
      PostcardSendId::new(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      "{}".to_string(),
      "{}".to_string(),
      PostcardType::Nenga,
      tomorrow,
      PostcardSendSource::Print,
      None,
      Utc::now(),
      Utc::now(),
      None,
    );
    assert_eq!(send.sent_on(), tomorrow);
  }

  #[test]
  fn from_persisted_for_update_rejects_future_when_date_changes() {
    let tomorrow = fixed_today() + chrono::Duration::days(1);
    let err = PostcardSend::from_persisted_for_update(
      PostcardSendId::new(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      "{}".to_string(),
      "{}".to_string(),
      PostcardType::Nenga,
      tomorrow,
      PostcardSendSource::Manual,
      None,
      Utc::now(),
      Utc::now(),
      None,
      fixed_today(),
      fixed_today(),
    )
    .expect_err("future change rejected");
    assert_eq!(err, PostcardSendError::FutureSentDate);
  }

  #[test]
  fn from_persisted_for_update_allows_unchanged_future_looking_date() {
    let tomorrow = fixed_today() + chrono::Duration::days(1);
    let send = PostcardSend::from_persisted_for_update(
      PostcardSendId::new(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      Uuid::new_v4(),
      "{}".to_string(),
      "{}".to_string(),
      PostcardType::Nenga,
      tomorrow,
      PostcardSendSource::Manual,
      None,
      Utc::now(),
      Utc::now(),
      None,
      tomorrow,
      fixed_today(),
    )
    .expect("unchanged future-looking date ok");
    assert_eq!(send.sent_on(), tomorrow);
  }

  #[test]
  fn mark_deleted_sets_deleted_at() {
    let mut send = base_create(fixed_today(), PostcardSendSource::Print, None).unwrap();
    send.mark_deleted();
    assert!(send.is_deleted());
  }

  #[test]
  fn source_enum_print_and_manual() {
    assert_eq!(PostcardSendSource::Print.as_str(), "print");
    assert_eq!(PostcardSendSource::Manual.as_str(), "manual");
  }
}
