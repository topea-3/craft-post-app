#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalCode {
  value: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostalCodeError {
  #[error("postal code must be 7 digits")]
  InvalidLength,
  #[error("postal code must contain digits only")]
  NonDigit,
}

impl PostalCode {
  pub fn new<S: Into<String>>(value: S) -> Result<Self, PostalCodeError> {
    let s = value.into();
    if s.len() != 7 {
      return Err(PostalCodeError::InvalidLength);
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
      return Err(PostalCodeError::NonDigit);
    }
    Ok(Self { value: s })
  }

  pub fn value(&self) -> &str {
    &self.value
  }

  /// 表示用 `"123-4567"` 形式
  pub fn formatted(&self) -> String {
    format!("{}-{}", &self.value[0..3], &self.value[3..7])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_valid_7_digit_code() {
    let pc = PostalCode::new("1234567").unwrap();
    assert_eq!(pc.value(), "1234567");
    assert_eq!(pc.formatted(), "123-4567");
  }

  #[test]
  fn rejects_invalid_length() {
    assert!(matches!(
      PostalCode::new("123456"),
      Err(PostalCodeError::InvalidLength)
    ));
    assert!(matches!(
      PostalCode::new("12345678"),
      Err(PostalCodeError::InvalidLength)
    ));
  }

  #[test]
  fn rejects_non_digits() {
    assert!(matches!(
      PostalCode::new("12a4567"),
      Err(PostalCodeError::NonDigit)
    ));
  }
}

