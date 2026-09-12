/// 送付履歴の登録経路
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostcardSendSource {
  Print,
  Manual,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostcardSendSourceError {
  #[error("invalid postcard send source: {0}")]
  Invalid(String),
}

impl PostcardSendSource {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Print => "print",
      Self::Manual => "manual",
    }
  }

  pub fn from_str(value: &str) -> Result<Self, PostcardSendSourceError> {
    match value {
      "print" => Ok(Self::Print),
      "manual" => Ok(Self::Manual),
      other => Err(PostcardSendSourceError::Invalid(other.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_known_sources() {
    assert_eq!(
      PostcardSendSource::from_str("print").unwrap(),
      PostcardSendSource::Print
    );
    assert_eq!(
      PostcardSendSource::from_str("manual").unwrap(),
      PostcardSendSource::Manual
    );
  }

  #[test]
  fn rejects_unknown() {
    assert!(PostcardSendSource::from_str("other").is_err());
    assert!(PostcardSendSource::from_str("").is_err());
  }

  #[test]
  fn as_str_roundtrip() {
    for s in [PostcardSendSource::Print, PostcardSendSource::Manual] {
      assert_eq!(PostcardSendSource::from_str(s.as_str()).unwrap(), s);
    }
  }
}
