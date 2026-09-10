use chrono::{NaiveDate, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::address::address_entry::AddressEntryId;
use crate::domain::address::address_entry_repository::{AddressEntryRepository, AddressRepositoryError};
use crate::domain::print::postcard_send::{PostcardSend, PostcardSendId};
use crate::domain::print::postcard_send_repository::{
  map_db_row_to_send, DbPostcardSendRow, PostcardSendAddressContext, PostcardSendRepository,
  PostcardSendRepositoryError, PostcardSendSearchQuery, PostcardSendSenderContext,
  PostcardSendWithContext, SendStatusFilter, SendStatusItem, SendStatusQuery, SortOrder,
};
use crate::domain::sender::sender_entry::SenderEntryId;
use crate::domain::sender::sender_entry_repository::{SenderEntryRepository, SenderRepositoryError};
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;
use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

pub struct SqlxPostcardSendRepository {
  pool: SqlitePool,
}

impl SqlxPostcardSendRepository {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

fn is_job_address_unique_violation(err: &sqlx::Error) -> bool {
  match err {
    sqlx::Error::Database(db_err) => {
      let msg = db_err.message();
      msg.contains("UNIQUE constraint failed")
        && (msg.contains("idx_postcard_sends_job_address")
          || msg.contains("print_job_id")
          || msg.contains("postcard_sends"))
    }
    _ => false,
  }
}

/// display_full_recipient() と同等の表示名を SQL で組み立てる式（受取一覧と同型）
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

fn sender_display_name_sql(se_alias: &str) -> String {
  format!(
    r#"TRIM(IFNULL({se}.primary_last, '') || ' ' || IFNULL({se}.primary_first, ''))"#,
    se = se_alias
  )
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

fn build_search_where_clause(query: &PostcardSendSearchQuery) -> String {
  let mut s = String::new();
  if !query.include_deleted {
    s.push_str(" AND ps.deleted_at IS NULL");
  }
  if query.year.is_some() {
    s.push_str(" AND ps.sent_on >= ? AND ps.sent_on <= ?");
  }
  if query.postcard_type.is_some() {
    s.push_str(" AND ps.postcard_type = ?");
  }
  if query.address_entry_id.is_some() {
    s.push_str(" AND ps.address_entry_id = ?");
  }
  if query.source.is_some() {
    s.push_str(" AND ps.source = ?");
  }
  if query.keyword.is_some() {
    let display = address_display_name_sql("ae");
    let sender_display = sender_display_name_sql("se");
    s.push_str(&format!(
      " AND (
          {display} LIKE ? ESCAPE '\\' OR
          IFNULL(se.label, '') LIKE ? ESCAPE '\\' OR
          {sender_display} LIKE ? ESCAPE '\\' OR
          IFNULL(ps.memo, '') LIKE ? ESCAPE '\\' OR
          (
            ae.id IS NULL AND (
              IFNULL(json_extract(ps.address_snapshot, '$.primary_last'), '') LIKE ? ESCAPE '\\' OR
              IFNULL(json_extract(ps.address_snapshot, '$.primary_first'), '') LIKE ? ESCAPE '\\'
            )
          ) OR
          (
            se.id IS NULL AND (
              IFNULL(json_extract(ps.sender_snapshot, '$.primary_last'), '') LIKE ? ESCAPE '\\' OR
              IFNULL(json_extract(ps.sender_snapshot, '$.primary_first'), '') LIKE ? ESCAPE '\\'
            )
          )
        )"
    ));
  }
  s
}

fn build_search_order_clause(sort_order: &SortOrder) -> String {
  let order = match sort_order {
    SortOrder::Asc => "ps.sent_on ASC, ps.id ASC",
    SortOrder::Desc => "ps.sent_on DESC, ps.id ASC",
  };
  format!(" ORDER BY {}", order)
}

fn map_row_to_db(row: &sqlx::sqlite::SqliteRow) -> DbPostcardSendRow {
  DbPostcardSendRow {
    id: row.get("id"),
    print_job_id: row.get("print_job_id"),
    address_entry_id: row.get("address_entry_id"),
    sender_entry_id: row.get("sender_entry_id"),
    sender_snapshot: row.get("sender_snapshot"),
    address_snapshot: row.get("address_snapshot"),
    postcard_type: row.get("postcard_type"),
    sent_on: row.get("sent_on"),
    source: row.get("source"),
    memo: row.get("memo"),
    created_at: row.get("created_at"),
    updated_at: row.get("updated_at"),
    deleted_at: row.get("deleted_at"),
  }
}

fn map_address_repo_error(err: AddressRepositoryError) -> PostcardSendRepositoryError {
  match err {
    AddressRepositoryError::Db(e) => PostcardSendRepositoryError::Db(e),
    AddressRepositoryError::InvalidPersistedData(s) => {
      PostcardSendRepositoryError::InvalidPersistedData(s)
    }
    AddressRepositoryError::NotFound => {
      PostcardSendRepositoryError::InvalidPersistedData("address entry not found".to_string())
    }
  }
}

fn map_sender_repo_error(err: SenderRepositoryError) -> PostcardSendRepositoryError {
  match err {
    SenderRepositoryError::Db(e) => PostcardSendRepositoryError::Db(e),
    SenderRepositoryError::InvalidPersistedData(s) => {
      PostcardSendRepositoryError::InvalidPersistedData(s)
    }
    SenderRepositoryError::NotFound => {
      PostcardSendRepositoryError::InvalidPersistedData("sender entry not found".to_string())
    }
    SenderRepositoryError::DuplicateActiveLabel => {
      PostcardSendRepositoryError::InvalidPersistedData("duplicate sender label".to_string())
    }
  }
}

fn address_context_from_entry(
  entry: &crate::domain::address::address_entry::AddressEntry,
) -> PostcardSendAddressContext {
  PostcardSendAddressContext {
    display_name: entry.display_full_recipient(),
    address_line: entry.address().to_single_line(),
    archived: entry.archived(),
  }
}

fn sender_context_from_entry(
  entry: &crate::domain::sender::sender_entry::SenderEntry,
) -> PostcardSendSenderContext {
  PostcardSendSenderContext {
    label: entry.label().value().to_string(),
    display_name: entry.display_full_name(),
    archived: entry.archived(),
  }
}

async fn resolve_contexts(
  pool: &SqlitePool,
  address_entry_id: Uuid,
  sender_entry_id: Uuid,
) -> Result<(Option<PostcardSendAddressContext>, Option<PostcardSendSenderContext>), PostcardSendRepositoryError>
{
  let address = SqlxAddressEntryRepository::new(pool.clone())
    .find_by_id(&AddressEntryId::from_uuid(address_entry_id))
    .await
    .map_err(map_address_repo_error)?
    .map(|e| address_context_from_entry(&e));
  let sender = SqlxSenderEntryRepository::new(pool.clone())
    .find_by_id(&SenderEntryId::from_uuid(sender_entry_id))
    .await
    .map_err(map_sender_repo_error)?
    .map(|e| sender_context_from_entry(&e));
  Ok((address, sender))
}

fn bind_search_params<'q>(
  mut q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
  query: &PostcardSendSearchQuery,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
  if let Some(year) = query.year {
    let start = format!("{year:04}-01-01");
    let end = format!("{year:04}-12-31");
    q = q.bind(start).bind(end);
  }
  if let Some(postcard_type) = query.postcard_type {
    q = q.bind(postcard_type.as_str());
  }
  if let Some(address_entry_id) = query.address_entry_id {
    q = q.bind(address_entry_id.to_string());
  }
  if let Some(source) = query.source {
    q = q.bind(source.as_str());
  }
  if let Some(keyword) = &query.keyword {
    let pattern = escape_like_pattern(keyword);
    // live address display, sender label, sender display, memo,
    // address snapshot last/first, sender snapshot last/first
    q = q
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern.clone())
      .bind(pattern);
  }
  q
}

fn year_range(year: i32) -> (String, String) {
  (
    format!("{year:04}-01-01"),
    format!("{year:04}-12-31"),
  )
}

fn build_status_where(query: &SendStatusQuery) -> (String, String) {
  let display = address_display_name_sql("ae");
  let address_line = address_line_sql("ae");

  let mut population = String::from(" ae.archived_at IS NULL ");

  if query.receipt_year.is_some() {
    population.push_str(
      r#"
      AND EXISTS (
        SELECT 1 FROM postcard_receipts pr
        WHERE pr.address_entry_id = ae.id
          AND pr.deleted_at IS NULL
          AND pr.address_entry_id IS NOT NULL
          AND pr.received_at >= ? AND pr.received_at <= ?
      )
    "#,
    );
  }

  if query.keyword.is_some() {
    population.push_str(&format!(
      " AND ({display} LIKE ? ESCAPE '\\' OR {address_line} LIKE ? ESCAPE '\\') "
    ));
  }

  let type_pred = if query.postcard_type.is_some() {
    " AND ps.postcard_type = ? "
  } else {
    ""
  };

  let status_pred = match query.status {
    SendStatusFilter::Sent => format!(
      r#"
      AND EXISTS (
        SELECT 1 FROM postcard_sends ps
        WHERE ps.address_entry_id = ae.id
          AND ps.deleted_at IS NULL
          AND ps.sent_on >= ? AND ps.sent_on <= ?
          {type_pred}
      )
    "#
    ),
    SendStatusFilter::Unsent => format!(
      r#"
      AND NOT EXISTS (
        SELECT 1 FROM postcard_sends ps
        WHERE ps.address_entry_id = ae.id
          AND ps.deleted_at IS NULL
          AND ps.sent_on >= ? AND ps.sent_on <= ?
          {type_pred}
      )
    "#
    ),
  };

  let agg_type_pred = if query.postcard_type.is_some() {
    " AND ps_agg.postcard_type = ? "
  } else {
    ""
  };

  let select_extra = format!(
    r#"
      (
        SELECT MAX(ps_agg.sent_on)
        FROM postcard_sends ps_agg
        WHERE ps_agg.address_entry_id = ae.id
          AND ps_agg.deleted_at IS NULL
          {agg_type_pred}
      ) AS last_sent_on,
      (
        SELECT COUNT(*)
        FROM postcard_sends ps_agg
        WHERE ps_agg.address_entry_id = ae.id
          AND ps_agg.deleted_at IS NULL
          {agg_type_pred}
      ) AS send_count
    "#
  );

  let where_clause = format!("{population} {status_pred}");
  (where_clause, select_extra)
}

fn bind_status_params<'q>(
  mut q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
  query: &SendStatusQuery,
  for_list: bool,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
  // list SELECT の相関サブクエリが WHERE より先に現れるため、集約 type を先に bind する。
  // count: receipt → keyword → status year → status type
  // list:  agg type×2 → receipt → keyword → status year → status type → limit/offset
  if for_list {
    if let Some(postcard_type) = query.postcard_type {
      q = q
        .bind(postcard_type.as_str())
        .bind(postcard_type.as_str());
    }
  }
  if let Some(receipt_year) = query.receipt_year {
    let (start, end) = year_range(receipt_year);
    q = q.bind(start).bind(end);
  }
  if let Some(keyword) = &query.keyword {
    let pattern = escape_like_pattern(keyword);
    q = q.bind(pattern.clone()).bind(pattern);
  }
  let (start, end) = year_range(query.year);
  q = q.bind(start).bind(end);
  if let Some(postcard_type) = query.postcard_type {
    q = q.bind(postcard_type.as_str());
  }
  q
}

#[async_trait::async_trait]
impl PostcardSendRepository for SqlxPostcardSendRepository {
  async fn create_batch(
    &self,
    sends: &[PostcardSend],
  ) -> Result<(), PostcardSendRepositoryError> {
    if sends.is_empty() {
      return Ok(());
    }

    let mut tx = self.pool.begin().await?;

    for send in sends {
      let id = send.id().as_uuid().to_string();
      let print_job_id = send.print_job_id().to_string();
      let address_entry_id = send.address_entry_id().to_string();
      let sender_entry_id = send.sender_entry_id().to_string();
      let sender_snapshot = send.sender_snapshot();
      let address_snapshot = send.address_snapshot();
      let postcard_type = send.postcard_type().as_str();
      let sent_on = send.sent_on().format("%Y-%m-%d").to_string();
      let source = send.source().as_str();
      let memo = send.memo().map(|m| m.text().to_string());
      let created_at = send.created_at().to_rfc3339();
      let updated_at = send.updated_at().to_rfc3339();
      let deleted_at = send.deleted_at().map(|t| t.to_rfc3339());

      let result = sqlx::query(
        r#"
          INSERT INTO postcard_sends (
            id, print_job_id, address_entry_id, sender_entry_id,
            sender_snapshot, address_snapshot, postcard_type,
            sent_on, source, memo, created_at, updated_at, deleted_at
          )
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
      )
      .bind(&id)
      .bind(&print_job_id)
      .bind(&address_entry_id)
      .bind(&sender_entry_id)
      .bind(sender_snapshot)
      .bind(address_snapshot)
      .bind(postcard_type)
      .bind(&sent_on)
      .bind(source)
      .bind(memo)
      .bind(&created_at)
      .bind(&updated_at)
      .bind(deleted_at)
      .execute(&mut *tx)
      .await;

      match result {
        Ok(_) => {}
        Err(e) if is_job_address_unique_violation(&e) => {
          drop(tx);
          return Err(PostcardSendRepositoryError::Conflict);
        }
        Err(e) => return Err(PostcardSendRepositoryError::Db(e)),
      }
    }

    tx.commit().await?;
    Ok(())
  }

  async fn list_address_entry_ids_for_print_job(
    &self,
    print_job_id: Uuid,
  ) -> Result<Vec<Uuid>, PostcardSendRepositoryError> {
    let job = print_job_id.to_string();
    let rows = sqlx::query(
      r#"
        SELECT address_entry_id
        FROM postcard_sends
        WHERE print_job_id = ? AND deleted_at IS NULL
      "#,
    )
    .bind(&job)
    .fetch_all(&self.pool)
    .await?;

    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
      let id_str: String = row.get("address_entry_id");
      let uuid = Uuid::parse_str(&id_str).map_err(|e| {
        PostcardSendRepositoryError::InvalidPersistedData(e.to_string())
      })?;
      ids.push(uuid);
    }
    Ok(ids)
  }

  async fn find_by_id(
    &self,
    id: &PostcardSendId,
  ) -> Result<Option<PostcardSendWithContext>, PostcardSendRepositoryError> {
    let id_str = id.as_uuid().to_string();
    let row = sqlx::query(
      r#"
        SELECT
          ps.id, ps.print_job_id, ps.address_entry_id, ps.sender_entry_id,
          ps.sender_snapshot, ps.address_snapshot, ps.postcard_type,
          ps.sent_on, ps.source, ps.memo,
          ps.created_at, ps.updated_at, ps.deleted_at
        FROM postcard_sends ps
        WHERE ps.id = ?
      "#,
    )
    .bind(&id_str)
    .fetch_optional(&self.pool)
    .await?;

    let Some(row) = row else {
      return Ok(None);
    };
    let send = map_db_row_to_send(map_row_to_db(&row))?;
    let (address, sender) =
      resolve_contexts(&self.pool, send.address_entry_id(), send.sender_entry_id()).await?;
    Ok(Some(PostcardSendWithContext {
      send,
      address,
      sender,
    }))
  }

  async fn search(
    &self,
    query: PostcardSendSearchQuery,
  ) -> Result<(Vec<PostcardSendWithContext>, i64), PostcardSendRepositoryError> {
    let where_extra = build_search_where_clause(&query);
    let order_clause = build_search_order_clause(&query.sort_order);

    let count_sql = format!(
      r#"
        SELECT COUNT(*) AS cnt
        FROM postcard_sends ps
        LEFT JOIN address_entries ae ON ps.address_entry_id = ae.id
        LEFT JOIN sender_entries se ON ps.sender_entry_id = se.id
        WHERE 1=1
        {where_extra}
      "#
    );

    let list_sql = format!(
      r#"
        SELECT
          ps.id, ps.print_job_id, ps.address_entry_id, ps.sender_entry_id,
          ps.sender_snapshot, ps.address_snapshot, ps.postcard_type,
          ps.sent_on, ps.source, ps.memo,
          ps.created_at, ps.updated_at, ps.deleted_at
        FROM postcard_sends ps
        LEFT JOIN address_entries ae ON ps.address_entry_id = ae.id
        LEFT JOIN sender_entries se ON ps.sender_entry_id = se.id
        WHERE 1=1
        {where_extra}
        {order_clause}
        LIMIT ? OFFSET ?
      "#
    );

    let mut count_q = sqlx::query(&count_sql);
    count_q = bind_search_params(count_q, &query);

    let mut list_q = sqlx::query(&list_sql);
    list_q = bind_search_params(list_q, &query);
    list_q = list_q
      .bind(query.pagination.limit)
      .bind(query.pagination.offset);

    let mut tx = self.pool.begin().await?;
    let total: i64 = count_q.fetch_one(&mut *tx).await?.get("cnt");
    let rows = list_q.fetch_all(&mut *tx).await?;
    tx.commit().await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
      let send = map_db_row_to_send(map_row_to_db(&row))?;
      let (address, sender) =
        resolve_contexts(&self.pool, send.address_entry_id(), send.sender_entry_id()).await?;
      items.push(PostcardSendWithContext {
        send,
        address,
        sender,
      });
    }
    Ok((items, total))
  }

  async fn update(
    &self,
    send: &PostcardSend,
    expected_updated_at: &str,
  ) -> Result<(), PostcardSendRepositoryError> {
    let id = send.id().as_uuid().to_string();
    let postcard_type = send.postcard_type().as_str();
    let sent_on = send.sent_on().format("%Y-%m-%d").to_string();
    let memo = send.memo().map(|m| m.text().to_string());
    let updated_at = send.updated_at().to_rfc3339();

    let result = sqlx::query(
      r#"
        UPDATE postcard_sends
        SET
          sent_on = ?,
          postcard_type = ?,
          memo = ?,
          updated_at = ?
        WHERE id = ? AND deleted_at IS NULL AND updated_at = ?
      "#,
    )
    .bind(&sent_on)
    .bind(postcard_type)
    .bind(memo)
    .bind(&updated_at)
    .bind(&id)
    .bind(expected_updated_at)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      let row = sqlx::query(
        "SELECT updated_at FROM postcard_sends WHERE id = ? AND deleted_at IS NULL",
      )
      .bind(&id)
      .fetch_optional(&self.pool)
      .await?;

      match row {
        None => return Err(PostcardSendRepositoryError::NotFound),
        Some(r) => {
          let current: String = r.get("updated_at");
          if current != expected_updated_at {
            return Err(PostcardSendRepositoryError::OptimisticLockConflict);
          }
          return Err(PostcardSendRepositoryError::NotFound);
        }
      }
    }
    Ok(())
  }

  async fn delete(&self, id: &PostcardSendId) -> Result<(), PostcardSendRepositoryError> {
    let now = Utc::now().to_rfc3339();
    let id_str = id.as_uuid().to_string();
    let result = sqlx::query(
      r#"
        UPDATE postcard_sends
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
      return Err(PostcardSendRepositoryError::NotFound);
    }
    Ok(())
  }

  async fn list_sent_years(&self) -> Result<Vec<i32>, PostcardSendRepositoryError> {
    let rows = sqlx::query(
      r#"
        SELECT DISTINCT CAST(substr(sent_on, 1, 4) AS INTEGER) AS y
        FROM postcard_sends
        WHERE deleted_at IS NULL
          AND length(sent_on) >= 4
        ORDER BY y DESC
      "#,
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.iter().map(|row| row.get::<i32, _>("y")).collect())
  }

  async fn search_send_status(
    &self,
    query: SendStatusQuery,
  ) -> Result<(Vec<SendStatusItem>, i64), PostcardSendRepositoryError> {
    let (where_clause, select_extra) = build_status_where(&query);
    let display = address_display_name_sql("ae");
    let address_line = address_line_sql("ae");

    let count_sql = format!(
      r#"
        SELECT COUNT(*) AS cnt
        FROM address_entries ae
        WHERE {where_clause}
      "#
    );

    let list_sql = format!(
      r#"
        SELECT
          ae.id AS address_entry_id,
          {display} AS display_name,
          {address_line} AS address_summary,
          {select_extra}
        FROM address_entries ae
        WHERE {where_clause}
        ORDER BY
          COALESCE(ae.primary_kana_last, ae.primary_last) ASC,
          COALESCE(ae.primary_kana_first, ae.primary_first) ASC,
          ae.id ASC
        LIMIT ? OFFSET ?
      "#
    );

    let mut count_q = sqlx::query(&count_sql);
    count_q = bind_status_params(count_q, &query, false);

    let mut list_q = sqlx::query(&list_sql);
    list_q = bind_status_params(list_q, &query, true);
    list_q = list_q
      .bind(query.pagination.limit)
      .bind(query.pagination.offset);

    let mut tx = self.pool.begin().await?;
    let total: i64 = count_q.fetch_one(&mut *tx).await?.get("cnt");
    let rows = list_q.fetch_all(&mut *tx).await?;
    tx.commit().await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
      let id_str: String = row.get("address_entry_id");
      let address_entry_id = Uuid::parse_str(&id_str)
        .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?;
      let display_name: String = row.get("display_name");
      let address_summary: String = row.get("address_summary");
      let last_sent_on_str: Option<String> = row.get("last_sent_on");
      let last_sent_on = match last_sent_on_str {
        Some(s) if !s.is_empty() => Some(
          NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| PostcardSendRepositoryError::InvalidPersistedData(e.to_string()))?,
        ),
        _ => None,
      };
      let send_count: i64 = row.get("send_count");
      items.push(SendStatusItem {
        address_entry_id,
        display_name,
        address_summary,
        last_sent_on,
        send_count,
      });
    }
    Ok((items, total))
  }
}

#[cfg(test)]
mod escape_tests {
  use super::escape_like_pattern;

  #[test]
  fn escapes_like_wildcards() {
    assert_eq!(escape_like_pattern("a%b_c\\d"), "%a\\%b\\_c\\\\d%");
  }
}
