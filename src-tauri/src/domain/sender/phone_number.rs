#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneNumber {
  value: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PhoneNumberError {
  #[error("phone number must not be empty")]
  Empty,
  #[error("phone number is too long (max {max} characters)")]
  TooLong { max: usize },
}

impl PhoneNumber {
  const MAX_LEN: usize = 32;

  pub fn new(value: String) -> Result<Self, PhoneNumberError> {
    if value.trim().is_empty() {
      return Err(PhoneNumberError::Empty);
    }
    if value.chars().count() > Self::MAX_LEN {
      return Err(PhoneNumberError::TooLong { max: Self::MAX_LEN });
    }
    Ok(Self { value })
  }

  pub fn value(&self) -> &str {
    &self.value
  }
}

