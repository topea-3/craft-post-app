//! API（Rust）側の `log` 出力制御。
//!
//! - **開発ビルド**（`debug_assertions`）: すべてのレベルを標準エラー出力へ。
//! - **本番ビルド**: 既定は出力なし。デバッグモード ON かつログフォルダ指定時のみ、DEBUG 以下をファイルへ。
//! - デバッグ状態とフォルダパスは **永続化しない**（プロセス内のみ）。
//! - CLI: `--api-debug` と `--api-debug-log-dir <path>`（または `=path`）で起動時からファイルログを有効化可能。
//! - ログフォルダは絶対パスかつ、ユーザープロファイル / AppData / 一時フォルダ配下に制限する。

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// フロントの設定画面向けスナップショット（メモリ上の状態のみ）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLogDebugSettingsDto {
  pub debug_enabled: bool,
  pub log_directory: Option<String>,
}

struct Inner {
  log_directory: Option<PathBuf>,
  debug_enabled: bool,
  writer: Option<BufWriter<std::fs::File>>,
}

/// Tauri の `State` で共有する API ロガー制御。
pub struct ApiLogger {
  is_dev: bool,
  inner: Mutex<Inner>,
}

struct SharedLogger(std::sync::Arc<ApiLogger>);

impl Log for SharedLogger {
  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    self.0.enabled(metadata)
  }

  fn log(&self, record: &Record<'_>) {
    self.0.log_record(record);
  }

  fn flush(&self) {
    self.0.flush_inner();
  }
}

impl ApiLogger {
  pub fn new(is_dev: bool, cli_debug: bool, cli_dir: Option<PathBuf>) -> Result<Self, String> {
    let mut inner = Inner {
      log_directory: None,
      debug_enabled: false,
      writer: None,
    };

    if !is_dev && cli_debug {
      match normalize_dir(cli_dir) {
        Some(raw) => match prepare_log_directory(raw) {
          Ok(dir) => match Self::open_log_writer(&dir) {
            Ok(writer) => {
              inner.log_directory = Some(dir);
              inner.writer = Some(writer);
              inner.debug_enabled = true;
            }
            Err(msg) => {
              eprintln!("[Craft Post] ログファイルを開けません: {msg} ファイルログは無効のまま起動します。");
            }
          },
          Err(msg) => {
            eprintln!("[Craft Post] --api-debug-log-dir が拒否されました: {msg} ファイルログは無効のまま起動します。");
          }
        },
        None => {
          eprintln!(
            "[Craft Post] --api-debug を使う場合は --api-debug-log-dir で出力フォルダを指定してください。ファイルログは無効のまま起動します。"
          );
        }
      }
    }

    Ok(Self {
      is_dev,
      inner: Mutex::new(inner),
    })
  }

  fn open_log_writer(dir: &Path) -> Result<BufWriter<std::fs::File>, String> {
    let name = format!(
      "craft-post-api-{}.log",
      Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = dir.join(name);
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&path)
      .map_err(|e| format!("ログファイルを開けません ({}): {}", path.display(), e))?;
    Ok(BufWriter::new(file))
  }

  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    if self.is_dev {
      return metadata.level() <= Level::Trace;
    }
    let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if inner.debug_enabled {
      metadata.level() <= Level::Debug
    } else {
      false
    }
  }

  fn log_record(&self, record: &Record<'_>) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!(
      "{} [{:<5}] {} — {}\n",
      ts,
      record.level(),
      record.target(),
      record.args()
    );
    if self.is_dev {
      if record.metadata().level() > Level::Trace {
        return;
      }
      let _ = std::io::stderr().write_all(line.as_bytes());
      return;
    }
    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if !inner.debug_enabled || record.metadata().level() > Level::Debug {
      return;
    }
    if let Some(w) = inner.writer.as_mut() {
      let _ = w.write_all(line.as_bytes());
    }
  }

  fn flush_inner(&self) {
    if self.is_dev {
      let _ = std::io::stderr().flush();
      return;
    }
    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(w) = inner.writer.as_mut() {
      let _ = w.flush();
    }
  }

  fn update_max_level(&self) {
    let max = if self.is_dev {
      LevelFilter::Trace
    } else {
      let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
      if inner.debug_enabled {
        LevelFilter::Debug
      } else {
        LevelFilter::Off
      }
    };
    log::set_max_level(max);
  }

  pub fn get_settings(&self) -> ApiLogDebugSettingsDto {
    if self.is_dev {
      return ApiLogDebugSettingsDto {
        debug_enabled: false,
        log_directory: None,
      };
    }
    let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    ApiLogDebugSettingsDto {
      debug_enabled: inner.debug_enabled,
      log_directory: inner
        .log_directory
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned()),
    }
  }

  /// ログ出力フォルダ（本番のみ有効）。`None` または空文字でクリア。
  pub fn set_debug_directory(&self, directory: Option<String>) -> Result<(), String> {
    if self.is_dev {
      return Ok(());
    }
    let path = match directory {
      None => None,
      Some(s) => {
        let t = s.trim();
        if t.is_empty() {
          None
        } else {
          Some(prepare_log_directory(PathBuf::from(t))?)
        }
      }
    };

    let need_writer = {
      let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
      inner.debug_enabled
    };

    // writer の準備が成功してから状態を更新する（失敗時に debug_enabled だけ残さない）
    let new_writer = if need_writer {
      match path.as_ref() {
        Some(dir) => Some(Self::open_log_writer(dir)?),
        None => None,
      }
    } else {
      None
    };

    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if inner.debug_enabled {
      if let Some(writer) = new_writer {
        inner.log_directory = path;
        inner.writer = Some(writer);
      } else {
        inner.log_directory = None;
        inner.writer = None;
        inner.debug_enabled = false;
      }
    } else {
      inner.log_directory = path;
    }
    drop(inner);
    self.update_max_level();
    Ok(())
  }

  /// 本番のみ。ON にするにはあらかじめ `set_debug_directory` でフォルダが必要。
  pub fn set_debug_enabled(&self, enabled: bool) -> Result<(), String> {
    if self.is_dev {
      return Ok(());
    }

    if enabled {
      let dir = {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.log_directory.clone().ok_or_else(|| {
          "ログ出力フォルダを指定してください。フォルダを設定してからデバッグモードを有効にしてください。"
            .to_string()
        })?
      };
      if dir.as_os_str().is_empty() {
        return Err("ログ出力フォルダを指定してください。".to_string());
      }
      // 作成・canonicalize・再検証が成功してから debug_enabled を立てる
      let prepared = prepare_log_directory(dir)?;
      let writer = Self::open_log_writer(&prepared)?;
      let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
      inner.log_directory = Some(prepared);
      inner.writer = Some(writer);
      inner.debug_enabled = true;
      drop(inner);
    } else {
      let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
      inner.writer = None;
      inner.debug_enabled = false;
      drop(inner);
    }
    self.update_max_level();
    Ok(())
  }

  pub fn install_global(self: std::sync::Arc<Self>) -> Result<(), SetLoggerError> {
    self.update_max_level();
    let shared = SharedLogger(std::sync::Arc::clone(&self));
    log::set_boxed_logger(Box::new(shared))?;
    Ok(())
  }
}

fn normalize_dir(cli_dir: Option<PathBuf>) -> Option<PathBuf> {
  cli_dir.and_then(|p| {
    if p.as_os_str().is_empty() {
      None
    } else {
      Some(p)
    }
  })
}

/// `.` / `..` を解決した絶対パスを返す（存在確認はしない）。
fn normalize_absolute_path(path: &Path) -> Result<PathBuf, String> {
  if !path.is_absolute() {
    return Err("ログフォルダは絶対パスで指定してください。".to_string());
  }

  let mut out = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Prefix(prefix) => out.push(prefix.as_os_str()),
      Component::RootDir => out.push(component.as_os_str()),
      Component::CurDir => {}
      Component::ParentDir => {
        if !out.pop() {
          return Err("不正なログフォルダパスです。".to_string());
        }
      }
      Component::Normal(seg) => out.push(seg),
    }
  }
  Ok(out)
}

fn path_key(path: &Path) -> String {
  let s = path.to_string_lossy();
  #[cfg(windows)]
  {
    let trimmed = s
      .strip_prefix(r"\\?\")
      .or_else(|| s.strip_prefix("//?/"))
      .unwrap_or(&s);
    trimmed.to_lowercase()
  }
  #[cfg(not(windows))]
  {
    s.into_owned()
  }
}

fn is_path_under(path: &Path, root: &Path) -> bool {
  let path_s = path_key(path);
  let root_s = path_key(root);
  if path_s == root_s {
    return true;
  }
  let sep = std::path::MAIN_SEPARATOR;
  path_s.starts_with(&(root_s + &sep.to_string()))
}

fn allowed_log_roots() -> Result<Vec<PathBuf>, String> {
  let mut roots = Vec::new();

  if let Some(home) = dirs_home() {
    roots.push(home);
  }
  roots.push(std::env::temp_dir());

  #[cfg(windows)]
  {
    for key in ["LOCALAPPDATA", "APPDATA"] {
      if let Ok(val) = std::env::var(key) {
        if !val.is_empty() {
          roots.push(PathBuf::from(val));
        }
      }
    }
  }

  if roots.is_empty() {
    return Err("許可されたログフォルダの基準パスを解決できません。".to_string());
  }
  Ok(roots)
}

fn dirs_home() -> Option<PathBuf> {
  #[cfg(windows)]
  {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
  }
  #[cfg(not(windows))]
  {
    std::env::var_os("HOME").map(PathBuf::from)
  }
}

/// 絶対パス化し、ユーザープロファイル / AppData / 一時フォルダ配下のみ許可する。
pub(crate) fn validate_log_directory(path: PathBuf) -> Result<PathBuf, String> {
  let normalized = normalize_absolute_path(&path)?;
  ensure_under_allowed_roots(&normalized)?;
  Ok(normalized)
}

fn ensure_under_allowed_roots(path: &Path) -> Result<(), String> {
  let roots = allowed_log_roots()?;
  if !roots.iter().any(|root| {
    let Ok(root_norm) = normalize_absolute_path(root) else {
      return false;
    };
    is_path_under(path, &root_norm)
  }) {
    return Err(
      "ログフォルダはユーザープロファイル、AppData、または一時フォルダ配下の絶対パスを指定してください。"
        .to_string(),
    );
  }
  Ok(())
}

/// 作成後に canonicalize し、許可ルート配下かを再検証する。
fn prepare_log_directory(path: PathBuf) -> Result<PathBuf, String> {
  let normalized = validate_log_directory(path)?;
  fs::create_dir_all(&normalized).map_err(|e| format!("ログフォルダを作成できません: {}", e))?;
  let canonical = normalized
    .canonicalize()
    .map_err(|e| format!("ログフォルダを解決できません: {}", e))?;

  let roots = allowed_log_roots()?;
  let under = roots.iter().any(|root| {
    let root_canon = root
      .canonicalize()
      .or_else(|_| normalize_absolute_path(root))
      .ok();
    root_canon
      .map(|r| is_path_under(&canonical, &r))
      .unwrap_or(false)
  });
  if !under {
    return Err(
      "ログフォルダはユーザープロファイル、AppData、または一時フォルダ配下の絶対パスを指定してください。"
        .to_string(),
    );
  }
  Ok(canonical)
}

/// CLI とビルド種別から API ロガーを初期化し、`log` クレートのグローバルロガーとして登録する。
pub fn init_api_logger() -> Result<std::sync::Arc<ApiLogger>, String> {
  let is_dev = cfg!(debug_assertions);
  let (cli_debug, cli_dir) = parse_api_log_cli_args();
  let logger = std::sync::Arc::new(ApiLogger::new(is_dev, cli_debug, cli_dir)?);
  logger
    .clone()
    .install_global()
    .map_err(|_| "ログの初期化に失敗しました（ロガーは一度だけ登録できます）。")?;
  Ok(logger)
}

pub(crate) fn parse_api_log_cli_args_from(args: &[String]) -> (bool, Option<PathBuf>) {
  let mut debug = false;
  let mut dir: Option<PathBuf> = None;
  let mut i = 0usize;
  while i < args.len() {
    let arg = args[i].as_str();
    match arg {
      "--api-debug" => {
        debug = true;
        i += 1;
      }
      a if a.starts_with("--api-debug-log-dir=") => {
        dir = Some(PathBuf::from(a.trim_start_matches("--api-debug-log-dir=")));
        i += 1;
      }
      "--api-debug-log-dir" => {
        if i + 1 < args.len() {
          dir = Some(PathBuf::from(&args[i + 1]));
          i += 2;
        } else {
          i += 1;
        }
      }
      _ => i += 1,
    }
  }
  (debug, dir)
}

fn parse_api_log_cli_args() -> (bool, Option<PathBuf>) {
  let args: Vec<String> = std::env::args().collect();
  parse_api_log_cli_args_from(&args)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_cli_equals_and_space_form() {
    let a = vec![
      "app".into(),
      "--api-debug".into(),
      "--api-debug-log-dir=C:\\tmp\\logs".into(),
    ];
    let (d, p) = parse_api_log_cli_args_from(&a);
    assert!(d);
    assert_eq!(p, Some(PathBuf::from("C:\\tmp\\logs")));

    let b = vec!["x".into(), "--api-debug-log-dir".into(), "/var/log/cp".into()];
    let (d2, p2) = parse_api_log_cli_args_from(&b);
    assert!(!d2);
    assert_eq!(p2, Some(PathBuf::from("/var/log/cp")));
  }

  #[test]
  fn reject_relative_log_directory() {
    let err = validate_log_directory(PathBuf::from("relative\\logs")).unwrap_err();
    assert!(err.contains("絶対パス"));
  }

  #[test]
  fn reject_path_outside_allowed_roots() {
    #[cfg(windows)]
    {
      let err = validate_log_directory(PathBuf::from(r"C:\Windows\Temp\..\System32\craft")).unwrap_err();
      assert!(
        err.contains("ユーザープロファイル") || err.contains("不正"),
        "unexpected: {err}"
      );
    }
    #[cfg(not(windows))]
    {
      let err = validate_log_directory(PathBuf::from("/etc/craft-post-logs")).unwrap_err();
      assert!(err.contains("ユーザープロファイル") || err.contains("一時"));
    }
  }

  #[test]
  fn accept_temp_subdirectory() {
    let dir = std::env::temp_dir().join("craft-post-api-log-test");
    let ok = validate_log_directory(dir.clone()).expect("temp subdir should be allowed");
    assert_eq!(ok, normalize_absolute_path(&dir).unwrap());
  }

  #[test]
  fn prepare_log_directory_accepts_temp_after_create() {
    let dir = std::env::temp_dir().join(format!(
      "craft-post-api-log-prepare-{}",
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let prepared = prepare_log_directory(dir.clone()).expect("prepare temp");
    assert!(prepared.exists());
    assert!(is_path_under(
      &prepared,
      &std::env::temp_dir()
        .canonicalize()
        .unwrap_or_else(|_| std::env::temp_dir())
    ));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn accept_user_profile_subdirectory() {
    let home = dirs_home().expect("home");
    let dir = home.join("craft-post-logs-test");
    let ok = validate_log_directory(dir.clone()).expect("home subdir should be allowed");
    assert_eq!(ok, normalize_absolute_path(&dir).unwrap());
  }

  #[test]
  fn normalize_collapses_dotdot() {
    let home = dirs_home().expect("home");
    let sneaky = home.join("a").join("..").join("craft-post-logs-test");
    let ok = validate_log_directory(sneaky).expect("normalized under home");
    assert_eq!(ok, normalize_absolute_path(&home.join("craft-post-logs-test")).unwrap());
  }
}
