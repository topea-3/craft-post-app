//! SQLite `LIKE` 用の共通エスケープ。

/// `%` / `_` / `\` をリテラルとして扱うパターンを返す（前後に `%` を付与）。
pub fn escape_like_pattern(keyword: &str) -> String {
  let mut escaped = String::with_capacity(keyword.len());
  for ch in keyword.chars() {
    match ch {
      '\\' | '%' | '_' => {
        escaped.push('\\');
        escaped.push(ch);
      }
      _ => escaped.push(ch),
    }
  }
  format!("%{escaped}%")
}

#[cfg(test)]
mod tests {
  use super::escape_like_pattern;

  #[test]
  fn escapes_percent_underscore_and_backslash() {
    assert_eq!(escape_like_pattern("a%b_c\\d"), "%a\\%b\\_c\\\\d%");
  }
}
