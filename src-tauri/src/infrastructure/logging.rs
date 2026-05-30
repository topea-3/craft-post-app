//! API（Rust）側の `log` 出力制御。
//!
//! - **開発ビルド**（`debug_assertions`）: すべてのレベルを標準エラー出力へ。
//! - **本番ビルド**: 既定は出力なし。デバッグモード ON かつログフォルダ指定時のみ、DEBUG 以下をファイルへ。
//! - デバッグ状態とフォルダパスは **永続化しない**（プロセス内のみ）。
//! - CLI: `--api-debug` と `--api-debug-log-dir <path>`（または `=path`）で起動時からファイルログを有効化可能。

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
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
        Some(dir) => {
          fs::create_dir_all(&dir).map_err(|e| format!("ログフォルダを作成できません: {}", e))?;
          inner.log_directory = Some(dir.clone());
          inner.writer = Some(Self::open_log_writer(&dir)?);
          inner.debug_enabled = true;
        }
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
    let path = directory.and_then(|s| {
      let t = s.trim();
      if t.is_empty() {
        None
      } else {
        Some(PathBuf::from(t))
      }
    });

    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    inner.log_directory = path.clone();

    if inner.debug_enabled {
      inner.writer = None;
      if let Some(ref dir) = path {
        fs::create_dir_all(dir).map_err(|e| format!("ログフォルダを作成できません: {}", e))?;
        inner.writer = Some(Self::open_log_writer(dir)?);
      } else {
        inner.debug_enabled = false;
      }
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
    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if enabled {
      let dir = inner.log_directory.as_ref().ok_or_else(|| {
        "ログ出力フォルダを指定してください。フォルダを設定してからデバッグモードを有効にしてください。"
          .to_string()
      })?;
      if dir.as_os_str().is_empty() {
        return Err("ログ出力フォルダを指定してください。".to_string());
      }
      fs::create_dir_all(dir).map_err(|e| format!("ログフォルダを作成できません: {}", e))?;
      inner.writer = Some(Self::open_log_writer(dir)?);
      inner.debug_enabled = true;
    } else {
      inner.writer = None;
      inner.debug_enabled = false;
    }
    drop(inner);
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
}
