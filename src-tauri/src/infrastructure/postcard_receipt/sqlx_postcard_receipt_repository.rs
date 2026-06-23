use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::domain::postcard_receipt::postcard_receipt::{PostcardReceipt, PostcardReceiptId};
use crate::domain::postcard_receipt::postcard_receipt_repository::{
  build_address_context_from_search_row, map_db_row_to_receipt, DbPostcardReceiptRow,
  DbPostcardReceiptSearchRow, PostcardReceiptRepository, PostcardReceiptRepositoryError,
  PostcardReceiptSearchQuery, PostcardReceiptWithContext, SortOrder,
};

pub struct SqlxPostcardReceiptRepository {
  pool: SqlitePool,
}

impl SqlxPostcardReceiptRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
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
    s.push_str(
      " AND (
          IFNULL(pr.sender_display_name, '') LIKE ? OR
          IFNULL(pr.memo, '') LIKE ? OR
          IFNULL(ae.primary_last, '') LIKE ? OR
          IFNULL(ae.primary_first, '') LIKE ?
        )",
    );
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
    address_primary_last: row.get("address_primary_last"),
    address_primary_first: row.get("address_primary_first"),
    address_honorific: row.get("address_honorific"),
    address_prefecture: row.get("address_prefecture"),
    address_city: row.get("address_city"),
    address_street: row.get("address_street"),
    address_building: row.get("address_building"),
    address_archived_at: row.get("address_archived_at"),
  }
}

fn map_search_row_to_context(row: DbPostcardReceiptSearchRow) -> Result<PostcardReceiptWithContext, PostcardReceiptRepositoryError> {
  let address = build_address_context_from_search_row(&row);
  let receipt = map_db_row_to_receipt(row.receipt)?;
  Ok(PostcardReceiptWithContext { receipt, address })
}

#[async_trait::async_trait]
impl PostcardReceiptRepository for SqlxPostcardReceiptRepository {
  async fn create(&self, receipt: &PostcardReceipt) -> Result<(), PostcardReceiptRepositoryError> {
    let id = receipt.id().as_uuid().to_string();
    let address_entry_id = receipt.address_entry_id().map(|u| u.to_string());
    let sender_display_name = receipt.sender_display_name().map(str::to_string);
    let received_at = receipt.received_at().format("%Y-%m-%d").to_string();
    let category = receipt.category().as_str().to_string();
    let memo = receipt.memo().map(|m| m.text().to_string());
    let deleted_at = receipt.deleted_at().map(|t| t.to_rfc3339());
    let created_at = receipt.created_at().to_rfc3339();
    let updated_at = receipt.updated_at().to_rfc3339();

    sqlx::query(
      r#"
        INSERT INTO postcard_receipts (
          id, address_entry_id, sender_display_name, received_at, category, memo,
          deleted_at, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&id)
    .bind(address_entry_id)
    .bind(sender_display_name)
    .bind(received_at)
    .bind(category)
    .bind(memo)
    .bind(deleted_at)
    .bind(created_at)
    .bind(updated_at)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn update(&self, receipt: &PostcardReceipt) -> Result<(), PostcardReceiptRepositoryError> {
    let id = receipt.id().as_uuid().to_string();
    let address_entry_id = receipt.address_entry_id().map(|u| u.to_string());
    let sender_display_name = receipt.sender_display_name().map(str::to_string);
    let received_at = receipt.received_at().format("%Y-%m-%d").to_string();
    let category = receipt.category().as_str().to_string();
    let memo = receipt.memo().map(|m| m.text().to_string());
    let deleted_at = receipt.deleted_at().map(|t| t.to_rfc3339());
    let created_at = receipt.created_at().to_rfc3339();
    let updated_at = receipt.updated_at().to_rfc3339();

    let result = sqlx::query(
      r#"
        UPDATE postcard_receipts
        SET
          address_entry_id = ?,
          sender_display_name = ?,
          received_at = ?,
          category = ?,
          memo = ?,
          deleted_at = ?,
          created_at = ?,
          updated_at = ?
        WHERE id = ?
      "#,
    )
    .bind(address_entry_id)
    .bind(sender_display_name)
    .bind(received_at)
    .bind(category)
    .bind(memo)
    .bind(deleted_at)
    .bind(created_at)
    .bind(updated_at)
    .bind(&id)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      return Err(PostcardReceiptRepositoryError::NotFound);
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
          pr.updated_at,
          ae.primary_last AS address_primary_last,
          ae.primary_first AS address_primary_first,
          ae.honorific AS address_honorific,
          ae.prefecture AS address_prefecture,
          ae.city AS address_city,
          ae.street AS address_street,
          ae.building AS address_building,
          ae.archived_at AS address_archived_at
        FROM postcard_receipts pr
        LEFT JOIN address_entries ae ON pr.address_entry_id = ae.id
        WHERE pr.id = ?
      "#,
    )
    .bind(&id_str)
    .fetch_optional(&self.pool)
    .await?;

    let Some(row) = row else {
      return Ok(None);
    };
    Ok(Some(map_search_row_to_context(map_search_row(&row))?))
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
          pr.updated_at,
          ae.primary_last AS address_primary_last,
          ae.primary_first AS address_primary_first,
          ae.honorific AS address_honorific,
          ae.prefecture AS address_prefecture,
          ae.city AS address_city,
          ae.street AS address_street,
          ae.building AS address_building,
          ae.archived_at AS address_archived_at
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
    list_q = list_q.bind(query.pagination.limit).bind(query.pagination.offset);

    let rows = list_q.fetch_all(&self.pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
      items.push(map_search_row_to_context(map_search_row(&row))?);
    }
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
    let pattern = format!("%{}%", keyword);
    q = q
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern);
  }
  q
}
