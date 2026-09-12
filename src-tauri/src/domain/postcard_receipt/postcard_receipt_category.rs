#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostcardReceiptCategory {
  Nenga,
  Mochu,
  Other,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostcardReceiptCategoryError {
  #[error("invalid postcard receipt category: {0}")]
  Invalid(String),
}

impl PostcardReceiptCategory {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Nenga => "nenga",
      Self::Mochu => "mochu",
      Self::Other => "other",
    }
  }

  pub fn display_name(&self) -> &'static str {
    match self {
      Self::Nenga => "年賀状",
      Self::Mochu => "喪中はがき",
      Self::Other => "その他",
    }
  }

  pub fn parse(value: &str) -> Result<Self, PostcardReceiptCategoryError> {
    match value {
      "nenga" => Ok(Self::Nenga),
      "mochu" => Ok(Self::Mochu),
      "other" => Ok(Self::Other),
      other => Err(PostcardReceiptCategoryError::Invalid(other.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_known_categories() {
    assert_eq!(PostcardReceiptCategory::parse("nenga").unwrap(), PostcardReceiptCategory::Nenga);
    assert_eq!(PostcardReceiptCategory::parse("mochu").unwrap(), PostcardReceiptCategory::Mochu);
    assert_eq!(PostcardReceiptCategory::parse("other").unwrap(), PostcardReceiptCategory::Other);
  }

  #[test]
  fn rejects_unknown_category() {
    assert!(PostcardReceiptCategory::parse("unknown").is_err());
  }
}
