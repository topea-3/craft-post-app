use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::print::postcard_type::PostcardType;
use crate::domain::print::print_layout_preference::{PrintLayoutPreference, PrintLayoutPreferenceId};
use crate::domain::print::print_layout_preference_repository::{
  PrintLayoutPreferenceRepository, PrintLayoutPreferenceRepositoryError,
};

pub struct SqlxPrintLayoutPreferenceRepository {
  pool: SqlitePool,
}

impl SqlxPrintLayoutPreferenceRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

fn map_row(row: &sqlx::sqlite::SqliteRow) -> Result<PrintLayoutPreference, PrintLayoutPreferenceRepositoryError> {
  let id = Uuid::parse_str(&row.get::<String, _>("id"))
    .map_err(|e| PrintLayoutPreferenceRepositoryError::InvalidPersistedData(e.to_string()))?;
  let postcard_type = PostcardType::from_str(&row.get::<String, _>("postcard_type"))
    .map_err(|e| PrintLayoutPreferenceRepositoryError::InvalidPersistedData(e.to_string()))?;
  let layer_id: String = row.get("layer_id");
  let offset_x_pt: f64 = row.get("offset_x_pt");
  let offset_y_pt: f64 = row.get("offset_y_pt");
  let updated_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
    .map_err(|e| PrintLayoutPreferenceRepositoryError::InvalidPersistedData(e.to_string()))?
    .with_timezone(&Utc);

  Ok(PrintLayoutPreference::from_persisted(
    PrintLayoutPreferenceId::from_uuid(id),
    postcard_type,
    layer_id,
    offset_x_pt,
    offset_y_pt,
    updated_at,
  ))
}

#[async_trait::async_trait]
impl PrintLayoutPreferenceRepository for SqlxPrintLayoutPreferenceRepository {
  async fn list_by_postcard_type(
    &self,
    postcard_type: PostcardType,
  ) -> Result<Vec<PrintLayoutPreference>, PrintLayoutPreferenceRepositoryError> {
    let rows = sqlx::query(
      r#"
        SELECT id, postcard_type, layer_id, offset_x_pt, offset_y_pt, updated_at
        FROM print_layout_preferences
        WHERE postcard_type = ?
        ORDER BY layer_id ASC
      "#,
    )
    .bind(postcard_type.as_str())
    .fetch_all(&self.pool)
    .await?;

    let mut prefs = Vec::with_capacity(rows.len());
    for row in &rows {
      prefs.push(map_row(row)?);
    }
    Ok(prefs)
  }

  async fn save_all(
    &self,
    postcard_type: PostcardType,
    preferences: &[PrintLayoutPreference],
  ) -> Result<(), PrintLayoutPreferenceRepositoryError> {
    let mut tx = self.pool.begin().await?;

    let type_str = postcard_type.as_str();
    for pref in preferences {
      debug_assert_eq!(pref.postcard_type(), postcard_type);
      let id = pref.id().as_uuid().to_string();
      let layer_id = pref.layer_id();
      let offset_x = pref.offset_x_pt();
      let offset_y = pref.offset_y_pt();
      let updated_at = pref.updated_at().to_rfc3339();

      sqlx::query(
        r#"
          INSERT INTO print_layout_preferences (
            id, postcard_type, layer_id, offset_x_pt, offset_y_pt, updated_at
          )
          VALUES (?, ?, ?, ?, ?, ?)
          ON CONFLICT(postcard_type, layer_id) DO UPDATE SET
            offset_x_pt = excluded.offset_x_pt,
            offset_y_pt = excluded.offset_y_pt,
            updated_at = excluded.updated_at
        "#,
      )
      .bind(&id)
      .bind(type_str)
      .bind(layer_id)
      .bind(offset_x)
      .bind(offset_y)
      .bind(&updated_at)
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;
    Ok(())
  }
}
