use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::domain::address::address_entry::AddressEntryId;
use crate::domain::address::address_entry_repository::{AddressEntryRepository, AddressRepositoryError};
use crate::domain::postcard_receipt::postcard_receipt::{PostcardReceipt, PostcardReceiptId};
use crate::domain::postcard_receipt::postcard_receipt_repository::{
  map_db_row_to_receipt, DbPostcardReceiptRow, DbPostcardReceiptSearchRow, PostcardReceiptAddressContext,
  PostcardReceiptRepository, PostcardReceiptRepositoryError, PostcardReceiptSearchQuery,
  PostcardReceiptWithContext, SortOrder,
};
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;

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
          IFNULL(ae.primary_first, '') LIKE ? OR
          IFNULL(ae.prefecture, '') || IFNULL(ae.city, '') || IFNULL(ae.street, '') || IFNULL(ae.building, '') LIKE ? OR
          EXISTS (
            SELECT 1 FROM address_co_recipients acr
            WHERE acr.address_entry_id = ae.id
              AND (
                IFNULL(acr.last, '') LIKE ? OR
                IFNULL(acr.first, '') LIKE ?
              )
          )
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
  }
}

fn map_search_row_to_receipt(row: DbPostcardReceiptSearchRow) -> Result<PostcardReceipt, PostcardReceiptRepositoryError> {
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

async fn resolve_address_context(
  pool: &SqlitePool,
  address_entry_id: Option<uuid::Uuid>,
) -> Result<Option<PostcardReceiptAddressContext>, PostcardReceiptRepositoryError> {
  let Some(id) = address_entry_id else {
    return Ok(None);
  };
  let entry = SqlxAddressEntryRepository::new(pool.clone())
    .find_by_id(&AddressEntryId::from_uuid(id))
    .await
    .map_err(map_address_repo_error)?;
  Ok(entry.map(|e| PostcardReceiptAddressContext {
    display_name: e.display_full_recipient(),
    address_line: e.address().to_single_line(),
    archived: e.archived(),
  }))
}

async fn with_resolved_address(
  pool: &SqlitePool,
  receipt: PostcardReceipt,
) -> Result<PostcardReceiptWithContext, PostcardReceiptRepositoryError> {
  let address = resolve_address_context(pool, receipt.address_entry_id()).await?;
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
    let updated_at = receipt.updated_at().to_rfc3339();

    // deleted_at / created_at は通常 UPDATE で触らない。
    // 削除済み行を復活させないため deleted_at IS NULL を必須にする。
    let result = sqlx::query(
      r#"
        UPDATE postcard_receipts
        SET
          address_entry_id = ?,
          sender_display_name = ?,
          received_at = ?,
          category = ?,
          memo = ?,
          updated_at = ?
        WHERE id = ? AND deleted_at IS NULL
      "#,
    )
    .bind(address_entry_id)
    .bind(sender_display_name)
    .bind(received_at)
    .bind(category)
    .bind(memo)
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
    Ok(Some(with_resolved_address(&self.pool, receipt).await?))
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
    list_q = list_q.bind(query.pagination.limit).bind(query.pagination.offset);

    let rows = list_q.fetch_all(&self.pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
      let receipt = map_search_row_to_receipt(map_search_row(&row))?;
      items.push(with_resolved_address(&self.pool, receipt).await?);
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
      .bind(pattern.clone()) // sender_display_name
      .bind(pattern.clone()) // memo
      .bind(pattern.clone()) // primary_last
      .bind(pattern.clone()) // primary_first
      .bind(pattern.clone()) // address line
      .bind(pattern.clone()) // co last
      .bind(pattern); // co first
  }
  q
}
