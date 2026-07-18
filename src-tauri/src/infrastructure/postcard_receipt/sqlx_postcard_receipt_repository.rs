use std::collections::HashMap;

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::address::address_entry::AddressEntryId;
use crate::domain::address::address_entry_repository::{AddressEntryRepository, AddressRepositoryError};
use crate::domain::postcard_receipt::postcard_receipt::{PostcardReceipt, PostcardReceiptId};
use crate::domain::postcard_receipt::postcard_receipt_repository::{
  map_db_row_to_receipt, DbPostcardReceiptRow, DbPostcardReceiptSearchRow, PostcardReceiptAddressContext,
  PostcardReceiptRepository, PostcardReceiptRepositoryError, PostcardReceiptSearchQuery,
  PostcardReceiptWithContext, SortOrder,
};
use crate::infrastructure::address::sqlx_address_entry_repository::{
  build_entries_with_co_recipients, SqlxAddressEntryRepository,
};

pub struct SqlxPostcardReceiptRepository {
  pool: SqlitePool,
}

impl SqlxPostcardReceiptRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

/// display_full_recipient() と同等の表示名を SQL で組み立てる式
fn address_display_name_sql(ae_alias: &str) -> String {
  format!(
    r#"TRIM(
      IFNULL({ae}.primary_last, '') || ' ' || IFNULL({ae}.primary_first, '') ||
      CASE
        WHEN EXISTS (
          SELECT 1 FROM address_co_recipients acr0
          WHERE acr0.address_entry_id = {ae}.id
        ) THEN '・' || IFNULL((
          SELECT group_concat(part, '・')
          FROM (
            SELECT
              CASE
                WHEN acr.last = {ae}.primary_last THEN acr.first
                ELSE TRIM(IFNULL(acr.last, '') || ' ' || IFNULL(acr.first, ''))
              END AS part
            FROM address_co_recipients acr
            WHERE acr.address_entry_id = {ae}.id
            ORDER BY acr.order_index
          )
        ), '')
        ELSE ''
      END ||
      CASE
        WHEN IFNULL({ae}.honorific, '') != '' AND {ae}.honorific != 'なし'
          THEN ' ' || {ae}.honorific
        ELSE ''
      END
    )"#,
    ae = ae_alias
  )
}

/// Address::to_single_line() と同等（建物前に空白）
fn address_line_sql(ae_alias: &str) -> String {
  format!(
    r#"(IFNULL({ae}.prefecture, '') || IFNULL({ae}.city, '') || IFNULL({ae}.street, '') ||
      CASE
        WHEN IFNULL({ae}.building, '') != '' THEN ' ' || {ae}.building
        ELSE ''
      END)"#,
    ae = ae_alias
  )
}

fn build_search_where_clause(query: &PostcardReceiptSearchQuery) -> String {
  let mut s = String::new();
  if !query.include_deleted {
    s.push_str(" AND pr.deleted_at IS NULL");
  }
  if query.year.is_some() {
    s.push_str(" AND pr.received_at >= ? AND pr.received_at <= ?");
  }
  if query.category.is_some() {
    s.push_str(" AND pr.category = ?");
  }
  if query.address_entry_id.is_some() {
    s.push_str(" AND pr.address_entry_id = ?");
  }
  if query.keyword.is_some() {
    let display = address_display_name_sql("ae");
    let address_line = address_line_sql("ae");
    s.push_str(&format!(
      " AND (
          IFNULL(pr.sender_display_name, '') LIKE ? ESCAPE '\\' OR
          IFNULL(pr.memo, '') LIKE ? ESCAPE '\\' OR
          {display} LIKE ? ESCAPE '\\' OR
          {address_line} LIKE ? ESCAPE '\\'
        )"
    ));
  }
  s
}

fn build_search_order_clause(sort_order: &SortOrder) -> String {
  let order = match sort_order {
    SortOrder::Asc => "pr.received_at ASC, pr.id ASC",
    SortOrder::Desc => "pr.received_at DESC, pr.id ASC",
  };
  format!(" ORDER BY {}", order)
}

fn escape_like_pattern(keyword: &str) -> String {
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

fn map_search_row(row: &sqlx::sqlite::SqliteRow) -> DbPostcardReceiptSearchRow {
  DbPostcardReceiptSearchRow {
    receipt: DbPostcardReceiptRow {
      id: row.get("id"),
      address_entry_id: row.get("address_entry_id"),
      sender_display_name: row.get("sender_display_name"),
      received_at: row.get("received_at"),
      category: row.get("category"),
      memo: row.get("memo"),
      deleted_at: row.get("deleted_at"),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    },
  }
}

fn map_search_row_to_receipt(
  row: DbPostcardReceiptSearchRow,
) -> Result<PostcardReceipt, PostcardReceiptRepositoryError> {
  map_db_row_to_receipt(row.receipt)
}

fn map_address_repo_error(err: AddressRepositoryError) -> PostcardReceiptRepositoryError {
  match err {
    AddressRepositoryError::Db(e) => PostcardReceiptRepositoryError::Db(e),
    AddressRepositoryError::InvalidPersistedData(s) => {
      PostcardReceiptRepositoryError::InvalidPersistedData(s)
    }
    AddressRepositoryError::NotFound => {
      PostcardReceiptRepositoryError::InvalidPersistedData("address entry not found".to_string())
    }
  }
}

fn context_from_entry(
  entry: &crate::domain::address::address_entry::AddressEntry,
) -> PostcardReceiptAddressContext {
  PostcardReceiptAddressContext {
    display_name: entry.display_full_recipient(),
    address_line: entry.address().to_single_line(),
    archived: entry.archived(),
  }
}

async fn resolve_address_context(
  pool: &SqlitePool,
  address_entry_id: Option<Uuid>,
) -> Result<Option<PostcardReceiptAddressContext>, PostcardReceiptRepositoryError> {
  let Some(id) = address_entry_id else {
    return Ok(None);
  };
  let entry = SqlxAddressEntryRepository::new(pool.clone())
    .find_by_id(&AddressEntryId::from_uuid(id))
    .await
    .map_err(map_address_repo_error)?;
  Ok(entry.map(|e| context_from_entry(&e)))
}

async fn load_address_contexts_batch(
  pool: &SqlitePool,
  address_ids: &[Uuid],
) -> Result<HashMap<Uuid, PostcardReceiptAddressContext>, PostcardReceiptRepositoryError> {
  let mut unique: Vec<String> = address_ids.iter().map(|u| u.to_string()).collect();
  unique.sort();
  unique.dedup();
  if unique.is_empty() {
    return Ok(HashMap::new());
  }

  let mut map = HashMap::new();
  const CHUNK: usize = 100;
  for chunk in unique.chunks(CHUNK) {
    let placeholders = chunk
      .iter()
      .enumerate()
      .map(|(i, _)| format!("?{}", i + 1))
      .collect::<Vec<_>>()
      .join(",");
    let sql = format!(
      r#"
        SELECT
          id, primary_last, primary_first, primary_kana_last, primary_kana_first,
          honorific, postal_code, prefecture, city, street, building, memo,
          archived_at, created_at, updated_at
        FROM address_entries
        WHERE id IN ({placeholders})
      "#
    );
    let mut q = sqlx::query(&sql);
    for id in chunk {
      q = q.bind(id);
    }
    let rows = q.fetch_all(pool).await?;
    let entries = build_entries_with_co_recipients(rows, pool)
      .await
      .map_err(map_address_repo_error)?;
    for entry in entries {
      map.insert(entry.id().as_uuid(), context_from_entry(&entry));
    }
  }
  Ok(map)
}

fn address_link_predicate_sql_for_create() -> &'static str {
  // ?1/?2 = address_entry_id, ?3/?4 = allow_archived_address_id（create では通常 None）
  r#"(
    ? IS NULL
    OR EXISTS (
      SELECT 1 FROM address_entries ae
      WHERE ae.id = ?
        AND (
          ae.archived_at IS NULL
          OR (? IS NOT NULL AND ae.id = ?)
        )
    )
  )"#
}

/// UPDATE 用: archive 例外は「現在行がまだその住所を指している」場合のみ許可する。
fn address_link_predicate_sql_for_update() -> &'static str {
  // ?1/?2 = 新しい address_entry_id
  // ?3/?4/?5 = allow_archived_address_id（現在リンク一致の確認にも使う）
  r#"(
    ? IS NULL
    OR EXISTS (
      SELECT 1 FROM address_entries ae
      WHERE ae.id = ?
        AND (
          ae.archived_at IS NULL
          OR (
            ? IS NOT NULL
            AND ae.id = ?
            AND postcard_receipts.address_entry_id = ?
          )
        )
    )
  )"#
}

#[async_trait::async_trait]
impl PostcardReceiptRepository for SqlxPostcardReceiptRepository {
  async fn create(
    &self,
    receipt: &PostcardReceipt,
    allow_archived_address_id: Option<Uuid>,
  ) -> Result<(), PostcardReceiptRepositoryError> {
    let id = receipt.id().as_uuid().to_string();
    let address_entry_id = receipt.address_entry_id().map(|u| u.to_string());
    let allow_archived = allow_archived_address_id.map(|u| u.to_string());
    let sender_display_name = receipt.sender_display_name().map(str::to_string);
    let received_at = receipt.received_at().format("%Y-%m-%d").to_string();
    let category = receipt.category().as_str().to_string();
    let memo = receipt.memo().map(|m| m.text().to_string());
    let deleted_at = receipt.deleted_at().map(|t| t.to_rfc3339());
    let created_at = receipt.created_at().to_rfc3339();
    let updated_at = receipt.updated_at().to_rfc3339();

    let sql = format!(
      r#"
        INSERT INTO postcard_receipts (
          id, address_entry_id, sender_display_name, received_at, category, memo,
          deleted_at, created_at, updated_at
        )
        SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?
        WHERE {pred}
      "#,
      pred = address_link_predicate_sql_for_create()
    );

    let result = sqlx::query(&sql)
      .bind(&id)
      .bind(&address_entry_id)
      .bind(sender_display_name)
      .bind(received_at)
      .bind(category)
      .bind(memo)
      .bind(deleted_at)
      .bind(created_at)
      .bind(updated_at)
      .bind(&address_entry_id)
      .bind(&address_entry_id)
      .bind(&allow_archived)
      .bind(&allow_archived)
      .execute(&self.pool)
      .await?;

    if result.rows_affected() == 0 {
      return Err(PostcardReceiptRepositoryError::AddressLinkRejected);
    }
    Ok(())
  }

  async fn update(
    &self,
    receipt: &PostcardReceipt,
    allow_archived_address_id: Option<Uuid>,
    expected_updated_at: &str,
  ) -> Result<(), PostcardReceiptRepositoryError> {
    let id = receipt.id().as_uuid().to_string();
    let address_entry_id = receipt.address_entry_id().map(|u| u.to_string());
    let allow_archived = allow_archived_address_id.map(|u| u.to_string());
    let sender_display_name = receipt.sender_display_name().map(str::to_string);
    let received_at = receipt.received_at().format("%Y-%m-%d").to_string();
    let category = receipt.category().as_str().to_string();
    let memo = receipt.memo().map(|m| m.text().to_string());
    let updated_at = receipt.updated_at().to_rfc3339();

    let sql = format!(
      r#"
        UPDATE postcard_receipts
        SET
          address_entry_id = ?,
          sender_display_name = ?,
          received_at = ?,
          category = ?,
          memo = ?,
          updated_at = ?
        WHERE id = ? AND deleted_at IS NULL AND updated_at = ?
          AND {pred}
      "#,
      pred = address_link_predicate_sql_for_update()
    );

    let result = sqlx::query(&sql)
      .bind(&address_entry_id)
      .bind(sender_display_name)
      .bind(received_at)
      .bind(category)
      .bind(memo)
      .bind(updated_at)
      .bind(&id)
      .bind(expected_updated_at)
      .bind(&address_entry_id)
      .bind(&address_entry_id)
      .bind(&allow_archived)
      .bind(&allow_archived)
      .bind(&allow_archived)
      .execute(&self.pool)
      .await?;

    if result.rows_affected() == 0 {
      let row = sqlx::query(
        "SELECT updated_at FROM postcard_receipts WHERE id = ? AND deleted_at IS NULL",
      )
      .bind(&id)
      .fetch_optional(&self.pool)
      .await?;

      match row {
        None => return Err(PostcardReceiptRepositoryError::NotFound),
        Some(r) => {
          let current: String = r.get("updated_at");
          if current != expected_updated_at {
            return Err(PostcardReceiptRepositoryError::Conflict);
          }
          return Err(PostcardReceiptRepositoryError::AddressLinkRejected);
        }
      }
    }
    Ok(())
  }

  async fn find_by_id(
    &self,
    id: &PostcardReceiptId,
  ) -> Result<Option<PostcardReceiptWithContext>, PostcardReceiptRepositoryError> {
    let id_str = id.as_uuid().to_string();
    let row = sqlx::query(
      r#"
        SELECT
          pr.id,
          pr.address_entry_id,
          pr.sender_display_name,
          pr.received_at,
          pr.category,
          pr.memo,
          pr.deleted_at,
          pr.created_at,
          pr.updated_at
        FROM postcard_receipts pr
        WHERE pr.id = ?
      "#,
    )
    .bind(&id_str)
    .fetch_optional(&self.pool)
    .await?;

    let Some(row) = row else {
      return Ok(None);
    };
    let receipt = map_search_row_to_receipt(map_search_row(&row))?;
    let address = resolve_address_context(&self.pool, receipt.address_entry_id()).await?;
    Ok(Some(PostcardReceiptWithContext { receipt, address }))
  }

  async fn search(
    &self,
    query: PostcardReceiptSearchQuery,
  ) -> Result<(Vec<PostcardReceiptWithContext>, i64), PostcardReceiptRepositoryError> {
    let where_extra = build_search_where_clause(&query);
    let order_clause = build_search_order_clause(&query.sort_order);

    let count_sql = format!(
      r#"
        SELECT COUNT(*) AS cnt
        FROM postcard_receipts pr
        LEFT JOIN address_entries ae ON pr.address_entry_id = ae.id
        WHERE 1=1
        {where_extra}
      "#,
      where_extra = where_extra
    );

    let mut count_q = sqlx::query(&count_sql);
    count_q = bind_search_params(count_q, &query);

    let total: i64 = count_q.fetch_one(&self.pool).await?.get("cnt");

    let list_sql = format!(
      r#"
        SELECT
          pr.id,
          pr.address_entry_id,
          pr.sender_display_name,
          pr.received_at,
          pr.category,
          pr.memo,
          pr.deleted_at,
          pr.created_at,
          pr.updated_at
        FROM postcard_receipts pr
        LEFT JOIN address_entries ae ON pr.address_entry_id = ae.id
        WHERE 1=1
        {where_extra}
        {order_clause}
        LIMIT ? OFFSET ?
      "#,
      where_extra = where_extra,
      order_clause = order_clause
    );

    let mut list_q = sqlx::query(&list_sql);
    list_q = bind_search_params(list_q, &query);
    list_q = list_q
      .bind(query.pagination.limit)
      .bind(query.pagination.offset);

    let rows = list_q.fetch_all(&self.pool).await?;
    let mut receipts = Vec::with_capacity(rows.len());
    for row in rows {
      receipts.push(map_search_row_to_receipt(map_search_row(&row))?);
    }

    let address_ids: Vec<Uuid> = receipts
      .iter()
      .filter_map(|r| r.address_entry_id())
      .collect();
    let address_map = load_address_contexts_batch(&self.pool, &address_ids).await?;

    let items = receipts
      .into_iter()
      .map(|receipt| {
        let address = receipt
          .address_entry_id()
          .and_then(|id| address_map.get(&id).cloned());
        PostcardReceiptWithContext { receipt, address }
      })
      .collect();

    Ok((items, total))
  }

  async fn delete(&self, id: &PostcardReceiptId) -> Result<(), PostcardReceiptRepositoryError> {
    let now = Utc::now().to_rfc3339();
    let id_str = id.as_uuid().to_string();
    let result = sqlx::query(
      r#"
        UPDATE postcard_receipts
        SET deleted_at = ?, updated_at = ?
        WHERE id = ? AND deleted_at IS NULL
      "#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&id_str)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      return Err(PostcardReceiptRepositoryError::NotFound);
    }
    Ok(())
  }

  async fn list_received_years(&self) -> Result<Vec<i32>, PostcardReceiptRepositoryError> {
    let rows = sqlx::query(
      r#"
        SELECT DISTINCT CAST(substr(received_at, 1, 4) AS INTEGER) AS y
        FROM postcard_receipts
        WHERE deleted_at IS NULL
          AND length(received_at) >= 4
        ORDER BY y DESC
      "#,
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.iter().map(|row| row.get::<i32, _>("y")).collect())
  }
}

fn bind_search_params<'q>(
  mut q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
  query: &PostcardReceiptSearchQuery,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
  if let Some(year) = query.year {
    let start = format!("{year:04}-01-01");
    let end = format!("{year:04}-12-31");
    q = q.bind(start).bind(end);
  }
  if let Some(category) = query.category {
    q = q.bind(category.as_str().to_string());
  }
  if let Some(address_entry_id) = query.address_entry_id {
    q = q.bind(address_entry_id.to_string());
  }
  if let Some(keyword) = &query.keyword {
    let pattern = escape_like_pattern(keyword);
    q = q
      .bind(pattern.clone()) // sender_display_name
      .bind(pattern.clone()) // memo
      .bind(pattern.clone()) // display name
      .bind(pattern); // address line
  }
  q
}

#[cfg(test)]
mod escape_tests {
  use super::escape_like_pattern;

  #[test]
  fn escapes_like_wildcards() {
    assert_eq!(escape_like_pattern("a%b_c\\d"), "%a\\%b\\_c\\\\d%");
  }
}
