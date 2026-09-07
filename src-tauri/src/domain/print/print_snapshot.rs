use serde::{Deserialize, Serialize};

use crate::domain::address::address_entry::AddressEntry;
use crate::domain::address::honorific::Honorific;
use crate::domain::address::person_name::PersonName;
use crate::domain::sender::sender_entry::SenderEntry;

/// 宛名連名の印刷上限（設計 FR-05）
pub const MAX_ADDRESS_CO_RECIPIENTS: usize = 3;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PrintSnapshotError {
  #[error("宛名の連名は最大{max}名までです（現在{actual}名）")]
  TooManyAddressCoRecipients { max: usize, actual: usize },
  #[error("差出人の連名は最大{max}名までです（現在{actual}名）")]
  TooManySenderCoRecipients { max: usize, actual: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoRecipientPrint {
  pub last: String,
  pub first: String,
  pub omit_last: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressPrintSnapshot {
  pub address_entry_id: String,
  pub postal_code: String,
  pub address_line1: String,
  pub address_line2: String,
  pub address_line3: String,
  pub primary_last: String,
  pub primary_first: String,
  pub co_recipients: Vec<CoRecipientPrint>,
  pub honorific_print: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderPrintSnapshot {
  pub sender_entry_id: String,
  pub postal_code: String,
  pub address_line1: String,
  pub address_line2: String,
  pub address_line3: String,
  pub primary_last: String,
  pub primary_first: String,
  pub co_recipients: Vec<CoRecipientPrint>,
}

fn honorific_print(honorific: Honorific) -> String {
  match honorific {
    Honorific::None => String::new(),
    other => other.as_str().to_string(),
  }
}

/// 住所1: 都道府県+市区町村 / 住所2: 町名番地 / 住所3: 建物名
fn address_lines(
  prefecture: &str,
  city: &str,
  street: &str,
  building: Option<&str>,
) -> (String, String, String) {
  let line1 = format!("{}{}", prefecture, city);
  let line2 = street.to_string();
  let line3 = building.map(str::trim).unwrap_or("").to_string();
  (line1, line2, line3)
}

fn co_recipients_print(
  primary: &PersonName,
  co_recipients: &[PersonName],
) -> Vec<CoRecipientPrint> {
  co_recipients
    .iter()
    .map(|co| CoRecipientPrint {
      last: co.last().to_string(),
      first: co.first().to_string(),
      omit_last: co.last() == primary.last(),
    })
    .collect()
}

impl AddressPrintSnapshot {
  pub fn from_address_entry(entry: &AddressEntry) -> Result<Self, PrintSnapshotError> {
    let co = entry.co_recipients();
    if co.len() > MAX_ADDRESS_CO_RECIPIENTS {
      return Err(PrintSnapshotError::TooManyAddressCoRecipients {
        max: MAX_ADDRESS_CO_RECIPIENTS,
        actual: co.len(),
      });
    }
    let primary = entry.primary_name();
    let addr = entry.address();
    let (address_line1, address_line2, address_line3) = address_lines(
      addr.prefecture(),
      addr.city(),
      addr.street(),
      addr.building(),
    );
    Ok(Self {
      address_entry_id: entry.id().as_uuid().to_string(),
      postal_code: entry.postal_code().formatted(),
      address_line1,
      address_line2,
      address_line3,
      primary_last: primary.last().to_string(),
      primary_first: primary.first().to_string(),
      co_recipients: co_recipients_print(primary, co),
      honorific_print: honorific_print(entry.honorific()),
    })
  }
}

impl SenderPrintSnapshot {
  pub fn from_sender_entry(entry: &SenderEntry) -> Result<Self, PrintSnapshotError> {
    let co = entry.co_recipients();
    if co.len() > SenderEntry::MAX_CO_RECIPIENTS {
      return Err(PrintSnapshotError::TooManySenderCoRecipients {
        max: SenderEntry::MAX_CO_RECIPIENTS,
        actual: co.len(),
      });
    }
    let primary = entry.primary_name();
    let addr = entry.address();
    let (address_line1, address_line2, address_line3) = address_lines(
      addr.prefecture(),
      addr.city(),
      addr.street(),
      addr.building(),
    );
    Ok(Self {
      sender_entry_id: entry.id().as_uuid().to_string(),
      postal_code: entry.postal_code().formatted(),
      address_line1,
      address_line2,
      address_line3,
      primary_last: primary.last().to_string(),
      primary_first: primary.first().to_string(),
      co_recipients: co_recipients_print(primary, co),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domain::address::address::Address;
  use crate::domain::address::postal_code::PostalCode;
  use crate::domain::sender::sender_label::SenderLabel;

  fn sample_address() -> Address {
    Address::new(
      "東京都".into(),
      "渋谷区".into(),
      "神南1-1-1".into(),
      Some("ビル3F".into()),
    )
    .unwrap()
  }

  fn sample_postal() -> PostalCode {
    PostalCode::new("1234567").unwrap()
  }

  fn name(last: &str, first: &str) -> PersonName {
    PersonName::new(last.into(), first.into(), None, None).unwrap()
  }

  #[test]
  fn honorific_none_prints_empty() {
    let entry = AddressEntry::create_new(
      name("山田", "太郎"),
      vec![],
      Honorific::None,
      sample_postal(),
      sample_address(),
      None,
    );
    let snap = AddressPrintSnapshot::from_address_entry(&entry).unwrap();
    assert_eq!(snap.honorific_print, "");
  }

  #[test]
  fn honorific_sama_prints_as_str() {
    let entry = AddressEntry::create_new(
      name("山田", "太郎"),
      vec![],
      Honorific::Sama,
      sample_postal(),
      sample_address(),
      None,
    );
    let snap = AddressPrintSnapshot::from_address_entry(&entry).unwrap();
    assert_eq!(snap.honorific_print, "様");
  }

  #[test]
  fn omit_last_when_same_family_name() {
    let entry = AddressEntry::create_new(
      name("山田", "太郎"),
      vec![name("山田", "花子"), name("佐藤", "次郎")],
      Honorific::Sama,
      sample_postal(),
      sample_address(),
      None,
    );
    let snap = AddressPrintSnapshot::from_address_entry(&entry).unwrap();
    assert!(snap.co_recipients[0].omit_last);
    assert!(!snap.co_recipients[1].omit_last);
  }

  #[test]
  fn address_lines_and_postal_format() {
    let entry = AddressEntry::create_new(
      name("山田", "太郎"),
      vec![],
      Honorific::Sama,
      sample_postal(),
      sample_address(),
      None,
    );
    let snap = AddressPrintSnapshot::from_address_entry(&entry).unwrap();
    assert_eq!(snap.postal_code, "123-4567");
    assert_eq!(snap.address_line1, "東京都渋谷区");
    assert_eq!(snap.address_line2, "神南1-1-1");
    assert_eq!(snap.address_line3, "ビル3F");
  }

  #[test]
  fn address_line3_empty_when_no_building() {
    let addr = Address::new(
      "北海道".into(),
      "テスト市".into(),
      "テスト町１−５−２".into(),
      None,
    )
    .unwrap();
    let entry = AddressEntry::create_new(
      name("誰々", "何某"),
      vec![],
      Honorific::Sama,
      sample_postal(),
      addr,
      None,
    );
    let snap = AddressPrintSnapshot::from_address_entry(&entry).unwrap();
    assert_eq!(snap.address_line1, "北海道テスト市");
    assert_eq!(snap.address_line2, "テスト町１−５−２");
    assert_eq!(snap.address_line3, "");
  }

  #[test]
  fn rejects_too_many_address_co_recipients() {
    let co = vec![
      name("山田", "一"),
      name("山田", "二"),
      name("山田", "三"),
      name("山田", "四"),
    ];
    let entry = AddressEntry::create_new(
      name("山田", "太郎"),
      co,
      Honorific::Sama,
      sample_postal(),
      sample_address(),
      None,
    );
    let err = AddressPrintSnapshot::from_address_entry(&entry).unwrap_err();
    assert_eq!(
      err,
      PrintSnapshotError::TooManyAddressCoRecipients {
        max: 3,
        actual: 4
      }
    );
  }

  #[test]
  fn sender_snapshot_builds() {
    let entry = SenderEntry::create_new(
      SenderLabel::new("自宅".into()).unwrap(),
      name("山田", "太郎"),
      vec![name("山田", "花子")],
      sample_postal(),
      sample_address(),
      None,
    )
    .unwrap();
    let snap = SenderPrintSnapshot::from_sender_entry(&entry).unwrap();
    assert_eq!(snap.primary_last, "山田");
    assert!(snap.co_recipients[0].omit_last);
  }
}
