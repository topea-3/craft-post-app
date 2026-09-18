use chrono::{Datelike, NaiveDate};

use crate::domain::print::postcard_type::PostcardType;

/// 送付年の決定結果（TOP-34）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendYearDecision {
  Year(i32),
  /// 年賀状かつ 2/1〜10/31（テスト印刷）。送付履歴は作成しない
  TestPrint,
}

/// はがき種別と基準日から送付年を決定する。
///
/// - 喪中: 基準日の西暦
/// - 年賀状: 11/1〜12/31 → year+1、1/1〜1/31 → year、それ以外 → TestPrint
pub fn decide_send_year(postcard_type: PostcardType, base_date: NaiveDate) -> SendYearDecision {
  match postcard_type {
    PostcardType::Mochu => SendYearDecision::Year(base_date.year()),
    PostcardType::Nenga => {
      let month = base_date.month();
      let year = base_date.year();
      match month {
        11 | 12 => SendYearDecision::Year(year + 1),
        1 => SendYearDecision::Year(year),
        _ => SendYearDecision::TestPrint,
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::NaiveDate;

  fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
  }

  #[test]
  fn mochu_uses_calendar_year() {
    assert_eq!(
      decide_send_year(PostcardType::Mochu, d(2026, 1, 1)),
      SendYearDecision::Year(2026)
    );
    assert_eq!(
      decide_send_year(PostcardType::Mochu, d(2025, 12, 31)),
      SendYearDecision::Year(2025)
    );
  }

  #[test]
  fn nenga_nov_dec_is_next_year() {
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2025, 11, 1)),
      SendYearDecision::Year(2026)
    );
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2025, 12, 31)),
      SendYearDecision::Year(2026)
    );
  }

  #[test]
  fn nenga_january_is_same_year() {
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2026, 1, 1)),
      SendYearDecision::Year(2026)
    );
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2026, 1, 31)),
      SendYearDecision::Year(2026)
    );
  }

  #[test]
  fn nenga_feb_to_oct_is_test_print() {
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2025, 10, 31)),
      SendYearDecision::TestPrint
    );
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2026, 2, 1)),
      SendYearDecision::TestPrint
    );
    assert_eq!(
      decide_send_year(PostcardType::Nenga, d(2026, 6, 15)),
      SendYearDecision::TestPrint
    );
  }
}
