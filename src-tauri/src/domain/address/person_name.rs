use crate::domain::address::honorific::Honorific;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonName {
  last: String,
  first: String,
  kana_last: Option<String>,
  kana_first: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PersonNameError {
  #[error("last name must not be empty")]
  EmptyLast,
  #[error("first name must not be empty")]
  EmptyFirst,
  #[error("{field} is too long (max {max} characters)")]
  TooLong { field: &'static str, max: usize },
}

impl PersonName {
  const MAX_LEN: usize = 128;

  pub fn new(
    last: String,
    first: String,
    kana_last: Option<String>,
    kana_first: Option<String>,
  ) -> Result<Self, PersonNameError> {
    if last.trim().is_empty() {
      return Err(PersonNameError::EmptyLast);
    }
    if first.trim().is_empty() {
      return Err(PersonNameError::EmptyFirst);
    }
    if last.chars().count() > Self::MAX_LEN {
      return Err(PersonNameError::TooLong {
        field: "last",
        max: Self::MAX_LEN,
      });
    }
    if first.chars().count() > Self::MAX_LEN {
      return Err(PersonNameError::TooLong {
        field: "first",
        max: Self::MAX_LEN,
      });
    }
    if let Some(ref k) = kana_last {
      if k.chars().count() > Self::MAX_LEN {
        return Err(PersonNameError::TooLong {
          field: "kana_last",
          max: Self::MAX_LEN,
        });
      }
    }
    if let Some(ref k) = kana_first {
      if k.chars().count() > Self::MAX_LEN {
        return Err(PersonNameError::TooLong {
          field: "kana_first",
          max: Self::MAX_LEN,
        });
      }
    }

    Ok(Self {
      last,
      first,
      kana_last,
      kana_first,
    })
  }

  pub fn last(&self) -> &str {
    &self.last
  }

  pub fn first(&self) -> &str {
    &self.first
  }

  pub fn kana_last(&self) -> Option<&str> {
    self.kana_last.as_deref()
  }

  pub fn kana_first(&self) -> Option<&str> {
    self.kana_first.as_deref()
  }

  /// 表示用の氏名（例: "山田 太郎"）
  pub fn display(&self) -> String {
    format!("{} {}", self.last, self.first)
  }

  /// 表示用の氏名（カナ）（例: "ヤマダ タロウ"）
  pub fn display_kana(&self) -> Option<String> {
    match (self.kana_last.as_ref(), self.kana_first.as_ref()) {
      (Some(last), Some(first)) => Some(format!("{} {}", last, first)),
      _ => None,
    }
  }

  /// 敬称付き表示名（例: "山田 太郎 様"）
  pub fn display_with_honorific(&self, honorific: &Honorific) -> String {
    let base = self.display();
    match honorific {
      Honorific::None => base,
      _ => format!("{} {}", base, honorific.as_str()),
    }
  }

  /// 連名表示ユーティリティ。
  ///
  /// 例: primary = "山田 太郎", co = ["山田 花子"] -> "山田 太郎・花子"
  pub fn join_recipients(primary: &PersonName, co_recipients: &[PersonName]) -> String {
    if co_recipients.is_empty() {
      return primary.display();
    }

    let mut parts = Vec::with_capacity(1 + co_recipients.len());
    parts.push(primary.display());

    for co in co_recipients {
      if co.last == primary.last {
        parts.push(co.first.clone());
      } else {
        parts.push(co.display());
      }
    }

    parts.join("・")
  }

  /// 連名＋敬称付き表示名。
  pub fn join_recipients_with_honorific(
    primary: &PersonName,
    co_recipients: &[PersonName],
    honorific: &Honorific,
  ) -> String {
    let base = Self::join_recipients(primary, co_recipients);
    match honorific {
      Honorific::None => base,
      _ => format!("{} {}", base, honorific.as_str()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domain::address::honorific::Honorific;

  fn name() -> PersonName {
    PersonName::new(
      "山田".to_string(),
      "太郎".to_string(),
      Some("ヤマダ".to_string()),
      Some("タロウ".to_string()),
    )
    .unwrap()
  }

  #[test]
  fn new_rejects_empty_last_or_first() {
    assert!(matches!(
      PersonName::new("".into(), "太郎".into(), None, None),
      Err(PersonNameError::EmptyLast)
    ));
    assert!(matches!(
      PersonName::new("山田".into(), " ".into(), None, None),
      Err(PersonNameError::EmptyFirst)
    ));
  }

  #[test]
  fn display_and_display_kana() {
    let n = name();
    assert_eq!(n.display(), "山田 太郎");
    assert_eq!(n.display_kana().as_deref(), Some("ヤマダ タロウ"));
  }

  #[test]
  fn display_with_honorific() {
    let n = name();
    assert_eq!(
      n.display_with_honorific(&Honorific::Sama),
      "山田 太郎 様"
    );
    assert_eq!(n.display_with_honorific(&Honorific::None), "山田 太郎");
  }

  #[test]
  fn join_recipients_same_last_name() {
    let primary = name();
    let co = PersonName::new(
      "山田".into(),
      "花子".into(),
      Some("ヤマダ".into()),
      Some("ハナコ".into()),
    )
    .unwrap();

    let s = PersonName::join_recipients(&primary, &[co]);
    assert_eq!(s, "山田 太郎・花子");
  }

  #[test]
  fn join_recipients_different_last_name() {
    let primary = name();
    let co = PersonName::new(
      "佐藤".into(),
      "花子".into(),
      Some("サトウ".into()),
      Some("ハナコ".into()),
    )
    .unwrap();

    let s = PersonName::join_recipients(&primary, &[co]);
    assert_eq!(s, "山田 太郎・佐藤 花子");
  }

  #[test]
  fn join_recipients_with_honorific_appends_suffix() {
    let primary = name();
    let co = PersonName::new(
      "山田".into(),
      "花子".into(),
      Some("ヤマダ".into()),
      Some("ハナコ".into()),
    )
    .unwrap();

    let s =
      PersonName::join_recipients_with_honorific(&primary, &[co], &Honorific::Sama);
    assert_eq!(s, "山田 太郎・花子 様");
  }
}

