use chrono::{DateTime, Utc};
use std::collections::HashMap;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::address::address_entry::AddressEntry;
use crate::domain::address::address_entry_repository::AddressRepositoryError;
use crate::domain::sender::sender_entry::{SenderEntry, SenderEntryId};
use crate::domain::sender::sender_entry_repository::{
  DbSenderCoRecipientRow, DbSenderEntryRow, Pagination, SenderEntryRepository, SenderRepositoryError,
};
use crate::infrastructure::address::sqlx_address_entry_repository::build_entries_with_co_recipients as build_address_entries_with_co_recipients;

pub struct SqlxSenderEntryRepository {
  pool: SqlitePool,
}

impl SqlxSenderEntryRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

fn map_address_repository_error(e: AddressRepositoryError) -> SenderRepositoryError {
  match e {
    AddressRepositoryError::Db(err) => SenderRepositoryError::Db(err),
    AddressRepositoryError::InvalidPersistedData(msg) => SenderRepositoryError::InvalidPersistedData(msg),
    AddressRepositoryError::NotFound => SenderRepositoryError::InvalidPersistedData(
      "address entry not found during linked list build".to_string(),
    ),
  }
}

#[async_trait::async_trait]
impl SenderEntryRepository for SqlxSenderEntryRepository {
  async fn create(&self, entry: &SenderEntry) -> Result<(), SenderRepositoryError> {
    let mut tx = self.pool.begin().await?;
    let id = entry.id().as_uuid().to_string();
    let primary = entry.primary_name();
    let postal = entry.postal_code().value().to_string();
    let addr = entry.address();
    let phone = entry.phone_number().map(|p| p.value().to_string());
    let created_at: DateTime<Utc> = entry.created_at();
    let updated_at: DateTime<Utc> = entry.updated_at();

    sqlx::query(
      r#"
        INSERT INTO sender_entries (
          id, label, primary_last, primary_first, primary_kana_last, primary_kana_first,
          postal_code, prefecture, city, street, building, phone_number, archived, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&id)
    .bind(entry.label().value())
    .bind(primary.last())
    .bind(primary.first())
    .bind(primary.kana_last())
    .bind(primary.kana_first())
    .bind(&postal)
    .bind(addr.prefecture())
    .bind(addr.city())
    .bind(addr.street())
    .bind(addr.building())
    .bind(phone)
    .bind(entry.archived())
    .bind(created_at.to_rfc3339())
    .bind(updated_at.to_rfc3339())
    .execute(&mut *tx)
    .await?;

    for (index, co) in entry.co_recipients().iter().enumerate() {
      let co_id = Uuid::new_v4().to_string();
      sqlx::query(
        r#"
          INSERT INTO sender_co_recipients (
            id, sender_entry_id, order_index, last, first, kana_last, kana_first, archived, created_at, updated_at
          )
          VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
      )
      .bind(co_id)
      .bind(&id)
      .bind(index as i64)
      .bind(co.last())
      .bind(co.first())
      .bind(co.kana_last())
      .bind(co.kana_first())
      .bind(created_at.to_rfc3339())
      .bind(updated_at.to_rfc3339())
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;
    Ok(())
  }

  async fn update(&self, entry: &SenderEntry) -> Result<(), SenderRepositoryError> {
    let mut tx = self.pool.begin().await?;
    let id = entry.id().as_uuid().to_string();
    let primary = entry.primary_name();
    let postal = entry.postal_code().value().to_string();
    let addr = entry.address();
    let phone = entry.phone_number().map(|p| p.value().to_string());
    let created_at: DateTime<Utc> = entry.created_at();
    let updated_at: DateTime<Utc> = entry.updated_at();

    sqlx::query(
      r#"
        UPDATE sender_entries
        SET
          label = ?,
          primary_last = ?, primary_first = ?, primary_kana_last = ?, primary_kana_first = ?,
          postal_code = ?, prefecture = ?, city = ?, street = ?, building = ?,
          phone_number = ?, archived = ?, created_at = ?, updated_at = ?
        WHERE id = ?
      "#,
    )
    .bind(entry.label().value())
    .bind(primary.last())
    .bind(primary.first())
    .bind(primary.kana_last())
    .bind(primary.kana_first())
    .bind(&postal)
    .bind(addr.prefecture())
    .bind(addr.city())
    .bind(addr.street())
    .bind(addr.building())
    .bind(phone)
    .bind(entry.archived())
    .bind(created_at.to_rfc3339())
    .bind(updated_at.to_rfc3339())
    .bind(&id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM sender_co_recipients WHERE sender_entry_id = ?")
      .bind(&id)
      .execute(&mut *tx)
      .await?;

    for (index, co) in entry.co_recipients().iter().enumerate() {
      let co_id = Uuid::new_v4().to_string();
      sqlx::query(
        r#"
          INSERT INTO sender_co_recipients (
            id, sender_entry_id, order_index, last, first, kana_last, kana_first, archived, created_at, updated_at
          )
          VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
      )
      .bind(co_id)
      .bind(&id)
      .bind(index as i64)
      .bind(co.last())
      .bind(co.first())
      .bind(co.kana_last())
      .bind(co.kana_first())
      .bind(created_at.to_rfc3339())
      .bind(updated_at.to_rfc3339())
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;
    Ok(())
  }

  async fn exists_active_label(
    &self,
    label: &str,
    exclude_id: Option<&SenderEntryId>,
  ) -> Result<bool, SenderRepositoryError> {
    let count: i64 = if let Some(id) = exclude_id {
      sqlx::query_scalar(
        r#"
          SELECT COUNT(*)
          FROM sender_entries
          WHERE archived = 0 AND label = ? AND id <> ?
        "#,
      )
      .bind(label)
      .bind(id.as_uuid().to_string())
      .fetch_one(&self.pool)
      .await?
    } else {
      sqlx::query_scalar(
        r#"
          SELECT COUNT(*)
          FROM sender_entries
          WHERE archived = 0 AND label = ?
        "#,
      )
      .bind(label)
      .fetch_one(&self.pool)
      .await?
    };
    Ok(count > 0)
  }

  async fn find_by_id(&self, id: &SenderEntryId) -> Result<Option<SenderEntry>, SenderRepositoryError> {
    let id_str = id.as_uuid().to_string();
    let row = sqlx::query(
      r#"
        SELECT
          id, label, primary_last, primary_first, primary_kana_last, primary_kana_first,
          postal_code, prefecture, city, street, building, phone_number, archived, created_at, updated_at
        FROM sender_entries
        WHERE id = ?
      "#,
    )
    .bind(&id_str)
    .fetch_optional(&self.pool)
    .await?;

    let Some(row) = row else {
      return Ok(None);
    };

    let entry_row = DbSenderEntryRow {
      id: row.get("id"),
      label: row.get("label"),
      primary_last: row.get("primary_last"),
      primary_first: row.get("primary_first"),
      primary_kana_last: row.get("primary_kana_last"),
      primary_kana_first: row.get("primary_kana_first"),
      postal_code: row.get("postal_code"),
      prefecture: row.get("prefecture"),
      city: row.get("city"),
      street: row.get("street"),
      building: row.get("building"),
      phone_number: row.get("phone_number"),
      archived: row.get::<i64, _>("archived") != 0,
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    };

    let co_rows = sqlx::query(
      r#"
        SELECT last, first, kana_last, kana_first
        FROM sender_co_recipients
        WHERE sender_entry_id = ?
        ORDER BY order_index ASC
      "#,
    )
    .bind(&id_str)
    .fetch_all(&self.pool)
    .await?;

    let co_recipients = co_rows
      .into_iter()
      .map(|r| DbSenderCoRecipientRow {
        last: r.get("last"),
        first: r.get("first"),
        kana_last: r.get("kana_last"),
        kana_first: r.get("kana_first"),
      })
      .collect();
    Ok(Some(entry_row.into_domain(co_recipients)?))
  }

  async fn list_linked_address_entries(
    &self,
    sender_entry_id: &SenderEntryId,
  ) -> Result<Vec<AddressEntry>, SenderRepositoryError> {
    let sender_id = sender_entry_id.as_uuid().to_string();
    let rows = sqlx::query(
      r#"
        SELECT
          ae.id,
          ae.primary_last,
          ae.primary_first,
          ae.primary_kana_last,
          ae.primary_kana_first,
          ae.honorific,
          ae.postal_code,
          ae.prefecture,
          ae.city,
          ae.street,
          ae.building,
          ae.memo,
          ae.archived,
          ae.created_at,
          ae.updated_at
        FROM sender_address_links sal
        JOIN address_entries ae ON ae.id = sal.address_entry_id AND ae.archived = 0
        WHERE sal.sender_entry_id = ?
        ORDER BY sal.updated_at DESC, sal.id ASC
      "#,
    )
    .bind(sender_id)
    .fetch_all(&self.pool)
    .await?;

    let entries = build_address_entries_with_co_recipients(rows, &self.pool)
      .await
      .map_err(map_address_repository_error)?;
    Ok(entries)
  }

  async fn list_active(&self, pagination: Pagination) -> Result<Vec<SenderEntry>, SenderRepositoryError> {
    let rows = sqlx::query(
      r#"
        SELECT
          id, label, primary_last, primary_first, primary_kana_last, primary_kana_first,
          postal_code, prefecture, city, street, building, phone_number, archived, created_at, updated_at
        FROM sender_entries
        WHERE archived = 0
        ORDER BY updated_at DESC, id ASC
        LIMIT ? OFFSET ?
      "#,
    )
    .bind(pagination.limit)
    .bind(pagination.offset)
    .fetch_all(&self.pool)
    .await?;

    build_entries_with_co_recipients(rows, &self.pool).await
  }

  async fn archive(&self, id: &SenderEntryId) -> Result<(), SenderRepositoryError> {
    let mut tx = self.pool.begin().await?;
    let id_str = id.as_uuid().to_string();
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE sender_entries SET archived = 1, updated_at = ? WHERE id = ?")
      .bind(now)
      .bind(&id_str)
      .execute(&mut *tx)
      .await?;
    if result.rows_affected() == 0 {
      return Err(SenderRepositoryError::NotFound);
    }
    sqlx::query("DELETE FROM sender_address_links WHERE sender_entry_id = ?")
      .bind(&id_str)
      .execute(&mut *tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }

  async fn find_sender_id_by_address_entry_id(
    &self,
    address_entry_id: Uuid,
  ) -> Result<Option<SenderEntryId>, SenderRepositoryError> {
    let row = sqlx::query(
      r#"
        SELECT sender_entry_id
        FROM sender_address_links
        WHERE address_entry_id = ?
        ORDER BY updated_at DESC
        LIMIT 1
      "#,
    )
    .bind(address_entry_id.to_string())
    .fetch_optional(&self.pool)
    .await?;
    let Some(row) = row else {
      return Ok(None);
    };
    let sender_id: String = row.get("sender_entry_id");
    let parsed = Uuid::parse_str(&sender_id)
      .map_err(|e| SenderRepositoryError::InvalidPersistedData(e.to_string()))?;
    Ok(Some(SenderEntryId::from_uuid(parsed)))
  }

  async fn replace_links_for_sender(
    &self,
    sender_entry_id: &SenderEntryId,
    address_entry_ids: &[Uuid],
  ) -> Result<(), SenderRepositoryError> {
    let mut tx = self.pool.begin().await?;
    let sender_id = sender_entry_id.as_uuid().to_string();
    let now = Utc::now().to_rfc3339();

    // 既存リンクのうち、今回含まれないものを削除。
    if address_entry_ids.is_empty() {
      sqlx::query("DELETE FROM sender_address_links WHERE sender_entry_id = ?")
        .bind(&sender_id)
        .execute(&mut *tx)
        .await?;
      tx.commit().await?;
      return Ok(());
    }

    let placeholders = address_entry_ids
      .iter()
      .enumerate()
      .map(|(i, _)| format!("?{}", i + 2))
      .collect::<Vec<_>>()
      .join(",");
    let delete_sql = format!(
      "DELETE FROM sender_address_links WHERE sender_entry_id = ?1 AND address_entry_id NOT IN ({})",
      placeholders
    );
    let mut delete_query = sqlx::query(&delete_sql).bind(&sender_id);
    for addr_id in address_entry_ids {
      delete_query = delete_query.bind(addr_id.to_string());
    }
    delete_query.execute(&mut *tx).await?;

    // 宛名ごとに既存リンクを一旦消し、当該 sender に再作成（宛名 -> sender を 0..1 に保つ）。
    for addr_id in address_entry_ids {
      let addr = addr_id.to_string();
      sqlx::query("DELETE FROM sender_address_links WHERE address_entry_id = ?")
        .bind(&addr)
        .execute(&mut *tx)
        .await?;
      sqlx::query(
        "INSERT INTO sender_address_links (id, sender_entry_id, address_entry_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
      )
      .bind(Uuid::new_v4().to_string())
      .bind(&sender_id)
      .bind(&addr)
      .bind(&now)
      .bind(&now)
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;
    Ok(())
  }

  async fn set_sender_for_address(
    &self,
    address_entry_id: Uuid,
    sender_entry_id: Option<&SenderEntryId>,
  ) -> Result<(), SenderRepositoryError> {
    let mut tx = self.pool.begin().await?;
    let addr = address_entry_id.to_string();

    // 宛名側は高々 1 件なので、既存リンクは先に削除して差し替える。
    sqlx::query("DELETE FROM sender_address_links WHERE address_entry_id = ?")
      .bind(&addr)
      .execute(&mut *tx)
      .await?;

    if let Some(sender_id) = sender_entry_id {
      let now = Utc::now().to_rfc3339();
      sqlx::query(
        "INSERT INTO sender_address_links (id, sender_entry_id, address_entry_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
      )
      .bind(Uuid::new_v4().to_string())
      .bind(sender_id.as_uuid().to_string())
      .bind(&addr)
      .bind(&now)
      .bind(&now)
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;
    Ok(())
  }
}

async fn build_entries_with_co_recipients(
  rows: Vec<sqlx::sqlite::SqliteRow>,
  pool: &SqlitePool,
) -> Result<Vec<SenderEntry>, SenderRepositoryError> {
  if rows.is_empty() {
    return Ok(Vec::new());
  }

  let mut entry_rows = Vec::with_capacity(rows.len());
  let mut ids: Vec<String> = Vec::with_capacity(rows.len());
  for row in rows {
    let id: String = row.get("id");
    ids.push(id.clone());
    entry_rows.push(DbSenderEntryRow {
      id,
      label: row.get("label"),
      primary_last: row.get("primary_last"),
      primary_first: row.get("primary_first"),
      primary_kana_last: row.get("primary_kana_last"),
      primary_kana_first: row.get("primary_kana_first"),
      postal_code: row.get("postal_code"),
      prefecture: row.get("prefecture"),
      city: row.get("city"),
      street: row.get("street"),
      building: row.get("building"),
      phone_number: row.get("phone_number"),
      archived: row.get::<i64, _>("archived") != 0,
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    });
  }

  // 連名を一括取得（IN 句のバインド数上限を避けるためチャンク分割）。
  const IN_CHUNK_SIZE: usize = 100;
  let mut grouped: HashMap<String, Vec<DbSenderCoRecipientRow>> = HashMap::new();

  for chunk in ids.chunks(IN_CHUNK_SIZE) {
    let placeholders = chunk
      .iter()
      .enumerate()
      .map(|(i, _)| format!("?{}", i + 1))
      .collect::<Vec<_>>()
      .join(",");
    let sql = format!(
      r#"
        SELECT sender_entry_id, last, first, kana_last, kana_first
        FROM sender_co_recipients
        WHERE sender_entry_id IN ({})
        ORDER BY sender_entry_id, order_index
      "#,
      placeholders
    );
    let mut q = sqlx::query(&sql);
    for id in chunk {
      q = q.bind(id);
    }
    let co_rows = q.fetch_all(pool).await?;
    for r in co_rows {
      let sender_entry_id: String = r.get("sender_entry_id");
      let row = DbSenderCoRecipientRow {
        last: r.get("last"),
        first: r.get("first"),
        kana_last: r.get("kana_last"),
        kana_first: r.get("kana_first"),
      };
      grouped.entry(sender_entry_id).or_default().push(row);
    }
  }

  let mut result = Vec::with_capacity(entry_rows.len());
  for er in entry_rows {
    let co = grouped.remove(&er.id).unwrap_or_default();
    result.push(er.into_domain(co)?);
  }
  Ok(result)
}

