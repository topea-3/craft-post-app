use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::address::address_entry::{AddressEntry, AddressEntryId};
use crate::domain::address::address_entry_repository::{
  AddressEntryRepository, AddressRepositoryError, AddressSearchQuery, DbAddressEntryRow,
  DbCoRecipientRow, Pagination, SortKey, SortOrder,
};

fn build_search_where_clause(query: &AddressSearchQuery) -> String {
  let mut s = String::new();
  if !query.include_archived {
    s.push_str(" AND archived = 0");
  }
  if query.keyword.is_some() {
    s.push_str(
      " AND (
          primary_last      LIKE ? OR
          primary_first     LIKE ? OR
          primary_kana_last LIKE ? OR
          primary_kana_first LIKE ? OR
          prefecture || city || street || IFNULL(building, '') LIKE ? OR
          IFNULL(memo, '') LIKE ?
        )",
    );
  }
  s
}

fn build_search_order_clause(sort_key: SortKey, sort_order: SortOrder) -> String {
  let order = match (sort_key, sort_order) {
    (SortKey::NameKana, SortOrder::Asc) => {
      "COALESCE(primary_kana_last, primary_last) ASC, COALESCE(primary_kana_first, primary_first) ASC, id ASC"
    }
    (SortKey::NameKana, SortOrder::Desc) => {
      "COALESCE(primary_kana_last, primary_last) DESC, COALESCE(primary_kana_first, primary_first) DESC, id ASC"
    }
    (SortKey::UpdatedAt, SortOrder::Asc) => "updated_at ASC, id ASC",
    (SortKey::UpdatedAt, SortOrder::Desc) => "updated_at DESC, id ASC",
  };
  format!(" ORDER BY {}", order)
}

pub struct SqlxAddressEntryRepository {
  pool: SqlitePool,
}

impl SqlxAddressEntryRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl AddressEntryRepository for SqlxAddressEntryRepository {
  async fn create(&self, entry: &AddressEntry) -> Result<(), AddressRepositoryError> {
    let mut tx = self.pool.begin().await?;

    let id = entry.id().as_uuid().to_string();
    let primary = entry.primary_name();
    let honorific = entry.honorific().as_str().to_string();
    let postal_code = entry.postal_code().value().to_string();
    let addr = entry.address();
    let memo_text = entry.memo().map(|m| m.text().to_string());
    let archived = entry.archived();
    let created_at: DateTime<Utc> = entry.created_at();
    let updated_at: DateTime<Utc> = entry.updated_at();

    sqlx::query(
      r#"
        INSERT INTO address_entries (
          id,
          primary_last,
          primary_first,
          primary_kana_last,
          primary_kana_first,
          honorific,
          postal_code,
          prefecture,
          city,
          street,
          building,
          memo,
          archived,
          created_at,
          updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&id)
    .bind(primary.last())
    .bind(primary.first())
    .bind(primary.kana_last())
    .bind(primary.kana_first())
    .bind(&honorific)
    .bind(&postal_code)
    .bind(addr.prefecture())
    .bind(addr.city())
    .bind(addr.street())
    .bind(addr.building())
    .bind(memo_text)
    .bind(archived)
    .bind(created_at.to_rfc3339())
    .bind(updated_at.to_rfc3339())
    .execute(&mut *tx)
    .await?;

    for (index, co) in entry.co_recipients().iter().enumerate() {
      let co_id = Uuid::new_v4().to_string();
      sqlx::query(
        r#"
          INSERT INTO address_co_recipients (
            id,
            address_entry_id,
            order_index,
            last,
            first,
            kana_last,
            kana_first,
            archived,
            created_at,
            updated_at
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

  async fn update(&self, entry: &AddressEntry) -> Result<(), AddressRepositoryError> {
    let mut tx = self.pool.begin().await?;

    let id = entry.id().as_uuid().to_string();
    let primary = entry.primary_name();
    let honorific = entry.honorific().as_str().to_string();
    let postal_code = entry.postal_code().value().to_string();
    let addr = entry.address();
    let memo_text = entry.memo().map(|m| m.text().to_string());
    let archived = entry.archived();
    let created_at: DateTime<Utc> = entry.created_at();
    let updated_at: DateTime<Utc> = entry.updated_at();

    sqlx::query(
      r#"
        UPDATE address_entries
        SET
          primary_last = ?,
          primary_first = ?,
          primary_kana_last = ?,
          primary_kana_first = ?,
          honorific = ?,
          postal_code = ?,
          prefecture = ?,
          city = ?,
          street = ?,
          building = ?,
          memo = ?,
          archived = ?,
          created_at = ?,
          updated_at = ?
        WHERE id = ?
      "#,
    )
    .bind(primary.last())
    .bind(primary.first())
    .bind(primary.kana_last())
    .bind(primary.kana_first())
    .bind(&honorific)
    .bind(&postal_code)
    .bind(addr.prefecture())
    .bind(addr.city())
    .bind(addr.street())
    .bind(addr.building())
    .bind(memo_text)
    .bind(archived)
    .bind(created_at.to_rfc3339())
    .bind(updated_at.to_rfc3339())
    .bind(&id)
    .execute(&mut *tx)
    .await?;

    // 連名は一旦削除してから挿入し直すシンプルな方針。
    sqlx::query("DELETE FROM address_co_recipients WHERE address_entry_id = ?")
      .bind(&id)
      .execute(&mut *tx)
      .await?;

    for (index, co) in entry.co_recipients().iter().enumerate() {
      let co_id = Uuid::new_v4().to_string();
      sqlx::query(
        r#"
          INSERT INTO address_co_recipients (
            id,
            address_entry_id,
            order_index,
            last,
            first,
            kana_last,
            kana_first,
            archived,
            created_at,
            updated_at
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

  async fn find_by_id(
    &self,
    id: &AddressEntryId,
  ) -> Result<Option<AddressEntry>, AddressRepositoryError> {
    let id_str = id.as_uuid().to_string();

    let row = sqlx::query(
      r#"
        SELECT
          id,
          primary_last,
          primary_first,
          primary_kana_last,
          primary_kana_first,
          honorific,
          postal_code,
          prefecture,
          city,
          street,
          building,
          memo,
          archived,
          created_at,
          updated_at
        FROM address_entries
        WHERE id = ?
      "#,
    )
    .bind(&id_str)
    .fetch_optional(&self.pool)
    .await?;

    let Some(row) = row else {
      return Ok(None);
    };

    let entry_row = DbAddressEntryRow {
      id: row.get("id"),
      primary_last: row.get("primary_last"),
      primary_first: row.get("primary_first"),
      primary_kana_last: row.get("primary_kana_last"),
      primary_kana_first: row.get("primary_kana_first"),
      honorific: row.get("honorific"),
      postal_code: row.get("postal_code"),
      prefecture: row.get("prefecture"),
      city: row.get("city"),
      street: row.get("street"),
      building: row.get("building"),
      memo: row.get("memo"),
      archived: row.get::<i64, _>("archived") != 0,
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    };

    let co_rows = sqlx::query(
      r#"
        SELECT
          id,
          address_entry_id,
          order_index,
          last,
          first,
          kana_last,
          kana_first,
          archived,
          created_at,
          updated_at
        FROM address_co_recipients
        WHERE address_entry_id = ?
        ORDER BY order_index ASC
      "#,
    )
    .bind(&id_str)
    .fetch_all(&self.pool)
    .await?;

    let co_recipients = co_rows
      .into_iter()
      .map(|r| DbCoRecipientRow {
        id: r.get("id"),
        address_entry_id: r.get("address_entry_id"),
        order_index: r.get("order_index"),
        last: r.get("last"),
        first: r.get("first"),
        kana_last: r.get("kana_last"),
        kana_first: r.get("kana_first"),
        archived: r.get::<i64, _>("archived") != 0,
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
      })
      .collect();

    let entry = entry_row.into_domain(co_recipients)?;
    Ok(Some(entry))
  }

  async fn list_active(
    &self,
    pagination: Pagination,
  ) -> Result<Vec<AddressEntry>, AddressRepositoryError> {
    let rows = sqlx::query(
      r#"
        SELECT
          id,
          primary_last,
          primary_first,
          primary_kana_last,
          primary_kana_first,
          honorific,
          postal_code,
          prefecture,
          city,
          street,
          building,
          memo,
          archived,
          created_at,
          updated_at
        FROM address_entries
        WHERE archived = 0
        ORDER BY
          COALESCE(primary_kana_last, primary_last) ASC,
          COALESCE(primary_kana_first, primary_first) ASC,
          id ASC
        LIMIT ? OFFSET ?
      "#,
    )
    .bind(pagination.limit)
    .bind(pagination.offset)
    .fetch_all(&self.pool)
    .await?;

    build_entries_with_co_recipients(rows, &self.pool).await
  }

  async fn search(
    &self,
    query: AddressSearchQuery,
  ) -> Result<(Vec<AddressEntry>, i64), AddressRepositoryError> {
    let where_clause = build_search_where_clause(&query);
    let order_clause = build_search_order_clause(query.sort_key.clone(), query.sort_order.clone());

    // 総件数取得（同じ WHERE で COUNT）
    let count_sql = format!("SELECT COUNT(*) AS cnt FROM address_entries WHERE 1 = 1 {}", where_clause);
    let mut count_q = sqlx::query(&count_sql);
    if let Some(keyword) = query.keyword.as_ref() {
      let kw = format!("%{}%", keyword);
      for _ in 0..6 {
        count_q = count_q.bind(kw.clone());
      }
    }
    let total: i64 = count_q
      .fetch_one(&self.pool)
      .await?
      .get("cnt");

    let mut sql = format!(
      r#"
        SELECT
          id,
          primary_last,
          primary_first,
          primary_kana_last,
          primary_kana_first,
          honorific,
          postal_code,
          prefecture,
          city,
          street,
          building,
          memo,
          archived,
          created_at,
          updated_at
        FROM address_entries
        WHERE 1 = 1
        {}
        {}
      "#,
      where_clause,
      order_clause,
    );

    if query.pagination.is_some() {
      sql.push_str(" LIMIT ? OFFSET ?");
    }

    let mut q = sqlx::query(&sql);

    if let Some(keyword) = query.keyword.as_ref() {
      let kw = format!("%{}%", keyword);
      for _ in 0..6 {
        q = q.bind(kw.clone());
      }
    }

    if let Some(p) = query.pagination {
      q = q.bind(p.limit).bind(p.offset);
    }

    let rows = q.fetch_all(&self.pool).await?;
    let entries = build_entries_with_co_recipients(rows, &self.pool).await?;
    Ok((entries, total))
  }

  async fn archive(&self, id: &AddressEntryId) -> Result<(), AddressRepositoryError> {
    let id_str = id.as_uuid().to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
      r#"
        UPDATE address_entries
        SET archived = 1, updated_at = ?
        WHERE id = ?
      "#,
    )
    .bind(now)
    .bind(id_str)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      return Err(AddressRepositoryError::NotFound);
    }

    Ok(())
  }
}

pub(crate) async fn build_entries_with_co_recipients(
  rows: Vec<sqlx::sqlite::SqliteRow>,
  pool: &SqlitePool,
) -> Result<Vec<AddressEntry>, AddressRepositoryError> {
  if rows.is_empty() {
    return Ok(Vec::new());
  }

  let mut entry_rows = Vec::with_capacity(rows.len());
  let mut ids: Vec<String> = Vec::with_capacity(rows.len());
  for row in rows {
    let id: String = row.get("id");
    ids.push(id.clone());
    entry_rows.push(DbAddressEntryRow {
      id,
      primary_last: row.get("primary_last"),
      primary_first: row.get("primary_first"),
      primary_kana_last: row.get("primary_kana_last"),
      primary_kana_first: row.get("primary_kana_first"),
      honorific: row.get("honorific"),
      postal_code: row.get("postal_code"),
      prefecture: row.get("prefecture"),
      city: row.get("city"),
      street: row.get("street"),
      building: row.get("building"),
      memo: row.get("memo"),
      archived: row.get::<i64, _>("archived") != 0,
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    });
  }

  // 連名を一括取得（IN 句のバインド数上限を避けるためチャンク分割）。
  const IN_CHUNK_SIZE: usize = 100;
  let mut grouped: HashMap<String, Vec<DbCoRecipientRow>> = HashMap::new();

  for chunk in ids.chunks(IN_CHUNK_SIZE) {
    let placeholders = chunk
      .iter()
      .enumerate()
      .map(|(i, _)| format!("?{}", i + 1))
      .collect::<Vec<_>>()
      .join(",");

    let sql = format!(
      r#"
        SELECT
          id,
          address_entry_id,
          order_index,
          last,
          first,
          kana_last,
          kana_first,
          archived,
          created_at,
          updated_at
        FROM address_co_recipients
        WHERE address_entry_id IN ({})
        ORDER BY address_entry_id, order_index
      "#,
      placeholders
    );

    let mut q = sqlx::query(&sql);
    for id in chunk {
      q = q.bind(id);
    }

    let co_rows_raw = q.fetch_all(pool).await?;
    for r in co_rows_raw {
      let entry_id: String = r.get("address_entry_id");
      let row = DbCoRecipientRow {
        id: r.get("id"),
        address_entry_id: entry_id.clone(),
        order_index: r.get("order_index"),
        last: r.get("last"),
        first: r.get("first"),
        kana_last: r.get("kana_last"),
        kana_first: r.get("kana_first"),
        archived: r.get::<i64, _>("archived") != 0,
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
      };
      grouped.entry(entry_id).or_default().push(row);
    }
  }

  let mut result = Vec::with_capacity(entry_rows.len());
  for er in entry_rows {
    let co = grouped.remove(&er.id).unwrap_or_default();
    let entry = er.into_domain(co)?;
    result.push(entry);
  }

  Ok(result)
}

