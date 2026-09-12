#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderLabel {
  value: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SenderLabelError {
  #[error("sender label must not be empty")]
  Empty,
  #[error("sender label is too long (max {max} characters)")]
  TooLong { max: usize },
}

impl SenderLabel {
  const MAX_LEN: usize = 250;

  pub fn new(value: String) -> Result<Self, SenderLabelError> {
    if value.trim().is_empty() {
      return Err(SenderLabelError::Empty);
    }
    if value.chars().count() > Self::MAX_LEN {
      return Err(SenderLabelError::TooLong { max: Self::MAX_LEN });
    }
    Ok(Self { value })
  }

  pub fn value(&self) -> &str {
    &self.value
  }
}

