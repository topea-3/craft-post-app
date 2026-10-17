use crate::infrastructure::db::DB_BUSY_MESSAGE;

#[derive(Debug, Clone)]
pub struct BackupError {
  pub code: &'static str,
  pub message: String,
}

impl BackupError {
  pub fn to_json(&self) -> String {
    serde_json::json!({
      "code": self.code,
      "message": self.message,
    })
    .to_string()
  }

  pub fn password_required() -> Self {
    Self {
      code: "password_required",
      message: "このバックアップはパスワードで保護されています。".to_string(),
    }
  }

  pub fn password_invalid() -> Self {
    Self {
      code: "password_invalid",
      message: "パスワードが違うか、ファイルが壊れています。".to_string(),
    }
  }

  pub fn password_unexpected() -> Self {
    Self {
      code: "password_unexpected",
      message: "このバックアップはパスワードで保護されていません。".to_string(),
    }
  }

  pub fn password_policy() -> Self {
    Self {
      code: "password_policy",
      message: "パスワードは 8 文字以上 128 文字以内で入力してください。".to_string(),
    }
  }

  pub fn not_backup() -> Self {
    Self {
      code: "not_backup",
      message: "Craft Post のバックアップファイルではありません。".to_string(),
    }
  }

  pub fn unsupported_format() -> Self {
    Self {
      code: "unsupported_format",
      message: "このバックアップ形式には対応していません。".to_string(),
    }
  }

  pub fn too_large() -> Self {
    Self {
      code: "unsupported_format",
      message: "バックアップが大きすぎます。".to_string(),
    }
  }

  pub fn schema_too_new() -> Self {
    Self {
      code: "schema_too_new",
      message: "このバックアップは新しいバージョンのアプリで作成されています。アプリを更新してから復元してください。"
        .to_string(),
    }
  }

  pub fn checksum_mismatch() -> Self {
    Self {
      code: "checksum_mismatch",
      message: "バックアップファイルが壊れています。".to_string(),
    }
  }

  pub fn busy() -> Self {
    Self {
      code: "busy",
      message: DB_BUSY_MESSAGE.to_string(),
    }
  }

  pub fn invalid_path() -> Self {
    Self {
      code: "invalid_path",
      message: "この場所には保存できません。".to_string(),
    }
  }

  pub fn io() -> Self {
    Self {
      code: "io",
      message: "ファイルを読み書きできませんでした。".to_string(),
    }
  }

  pub fn reopen_failed() -> Self {
    Self {
      code: "io",
      message: "データベースを開き直せませんでした。アプリを再起動してください。".to_string(),
    }
  }
}
