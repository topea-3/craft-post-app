#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memo {
  text: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MemoError {
  #[error("memo is too long (max {max} characters)")]
  TooLong { max: usize },
}

impl Memo {
  const MAX_LEN: usize = 1000;

  pub fn new<S: Into<String>>(text: S) -> Result<Self, MemoError> {
    let t = text.into();
    if t.chars().count() > Self::MAX_LEN {
      return Err(MemoError::TooLong { max: Self::MAX_LEN });
    }
    Ok(Self { text: t })
  }

  pub fn text(&self) -> &str {
    &self.text
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_within_limit() {
    let m = Memo::new("メモです").unwrap();
    assert_eq!(m.text(), "メモです");
  }

  #[test]
  fn rejects_too_long() {
    let long = "a".repeat(Memo::MAX_LEN + 1);
    assert!(matches!(
      Memo::new(long),
      Err(MemoError::TooLong { max }) if max == Memo::MAX_LEN
    ));
  }
}

