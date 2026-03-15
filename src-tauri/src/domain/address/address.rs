#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
  prefecture: String,
  city: String,
  street: String,
  building: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AddressError {
  #[error("{field} must not be empty")]
  EmptyField { field: &'static str },
  #[error("{field} is too long (max {max} characters)")]
  TooLong { field: &'static str, max: usize },
}

impl Address {
  const MAX_LEN: usize = 256;

  pub fn new(
    prefecture: String,
    city: String,
    street: String,
    building: Option<String>,
  ) -> Result<Self, AddressError> {
    if prefecture.trim().is_empty() {
      return Err(AddressError::EmptyField { field: "prefecture" });
    }
    if city.trim().is_empty() {
      return Err(AddressError::EmptyField { field: "city" });
    }
    if street.trim().is_empty() {
      return Err(AddressError::EmptyField { field: "street" });
    }

    let check_len = |value: &str, field: &'static str| -> Result<(), AddressError> {
      if value.chars().count() > Self::MAX_LEN {
        Err(AddressError::TooLong {
          field,
          max: Self::MAX_LEN,
        })
      } else {
        Ok(())
      }
    };

    check_len(&prefecture, "prefecture")?;
    check_len(&city, "city")?;
    check_len(&street, "street")?;
    if let Some(ref b) = building {
      check_len(b, "building")?;
    }

    Ok(Self {
      prefecture,
      city,
      street,
      building,
    })
  }

  pub fn prefecture(&self) -> &str {
    &self.prefecture
  }

  pub fn city(&self) -> &str {
    &self.city
  }

  pub fn street(&self) -> &str {
    &self.street
  }

  pub fn building(&self) -> Option<&str> {
    self.building.as_deref()
  }

  /// 1 行表現（例: "東京都渋谷区神南 1-1-1 ○○ビル 3F"）
  pub fn to_single_line(&self) -> String {
    match self.building.as_ref() {
      Some(b) if !b.trim().is_empty() => {
        format!("{}{}{} {}", self.prefecture, self.city, self.street, b)
      }
      _ => format!("{}{}{}", self.prefecture, self.city, self.street),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_empty_required_fields() {
    assert!(matches!(
      Address::new("".into(), "渋谷区".into(), "神南 1-1-1".into(), None),
      Err(AddressError::EmptyField { field }) if field == "prefecture"
    ));
    assert!(matches!(
      Address::new("東京都".into(), "".into(), "神南 1-1-1".into(), None),
      Err(AddressError::EmptyField { field }) if field == "city"
    ));
    assert!(matches!(
      Address::new("東京都".into(), "渋谷区".into(), " ".into(), None),
      Err(AddressError::EmptyField { field }) if field == "street"
    ));
  }

  #[test]
  fn to_single_line_with_and_without_building() {
    let addr = Address::new(
      "東京都".into(),
      "渋谷区".into(),
      "神南 1-1-1".into(),
      Some("○○ビル 3F".into()),
    )
    .unwrap();
    assert_eq!(
      addr.to_single_line(),
      "東京都渋谷区神南 1-1-1 ○○ビル 3F"
    );

    let addr_no_building =
      Address::new("東京都".into(), "渋谷区".into(), "神南 1-1-1".into(), None).unwrap();
    assert_eq!(addr_no_building.to_single_line(), "東京都渋谷区神南 1-1-1");
  }
}

