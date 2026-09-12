#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Honorific {
  Sama,
  Onchu,
  GokazokuSama,
  None,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HonorificError {
  #[error("invalid honorific: {0}")]
  InvalidValue(String),
}

impl Honorific {
  pub fn from_str(s: &str) -> Result<Self, HonorificError> {
    match s {
      "様" => Ok(Honorific::Sama),
      "御中" => Ok(Honorific::Onchu),
      "ご家族様" => Ok(Honorific::GokazokuSama),
      "なし" => Ok(Honorific::None),
      other => Err(HonorificError::InvalidValue(other.to_string())),
    }
  }

  pub fn as_str(&self) -> &str {
    match self {
      Honorific::Sama => "様",
      Honorific::Onchu => "御中",
      Honorific::GokazokuSama => "ご家族様",
      Honorific::None => "なし",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str_accepts_only_defined_values() {
    assert_eq!(Honorific::from_str("様").unwrap(), Honorific::Sama);
    assert_eq!(Honorific::from_str("御中").unwrap(), Honorific::Onchu);
    assert_eq!(
      Honorific::from_str("ご家族様").unwrap(),
      Honorific::GokazokuSama
    );
    assert_eq!(Honorific::from_str("なし").unwrap(), Honorific::None);

    assert!(matches!(
      Honorific::from_str("殿"),
      Err(HonorificError::InvalidValue(_))
    ));
  }

  #[test]
  fn as_str_roundtrip() {
    for h in [
      Honorific::Sama,
      Honorific::Onchu,
      Honorific::GokazokuSama,
      Honorific::None,
    ] {
      let s = h.as_str();
      let parsed = Honorific::from_str(s).unwrap();
      assert_eq!(h, parsed);
    }
  }
}

