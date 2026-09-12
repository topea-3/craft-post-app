/// 印刷対象のはがき種別（受取の `other` は含まない）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostcardType {
  Nenga,
  Mochu,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostcardTypeError {
  #[error("invalid postcard type: {0}")]
  Invalid(String),
}

impl PostcardType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Nenga => "nenga",
      Self::Mochu => "mochu",
    }
  }

  pub fn from_str(value: &str) -> Result<Self, PostcardTypeError> {
    match value {
      "nenga" => Ok(Self::Nenga),
      "mochu" => Ok(Self::Mochu),
      other => Err(PostcardTypeError::Invalid(other.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_known_types() {
    assert_eq!(PostcardType::from_str("nenga").unwrap(), PostcardType::Nenga);
    assert_eq!(PostcardType::from_str("mochu").unwrap(), PostcardType::Mochu);
  }

  #[test]
  fn rejects_other_and_unknown() {
    assert!(PostcardType::from_str("other").is_err());
    assert!(PostcardType::from_str("unknown").is_err());
  }

  #[test]
  fn as_str_roundtrip() {
    for t in [PostcardType::Nenga, PostcardType::Mochu] {
      assert_eq!(PostcardType::from_str(t.as_str()).unwrap(), t);
    }
  }
}
