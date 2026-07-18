mod domain;
mod infrastructure;
#[cfg(test)]
mod command_tests;

use std::sync::Arc;

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tauri::{Manager, State};
use uuid::Uuid;

use crate::infrastructure::logging::{ApiLogDebugSettingsDto, ApiLogger};

use crate::domain::address::address::Address;
use crate::domain::address::address_entry::{AddressEntry, AddressEntryId};
use crate::domain::address::address_entry_repository::{
  AddressEntryRepository, AddressSearchQuery, Pagination, SortKey, SortOrder,
};
use crate::domain::address::honorific::Honorific;
use crate::domain::address::memo::Memo;
use crate::domain::address::person_name::PersonName;
use crate::domain::address::postal_code::PostalCode;
use crate::domain::postcard_receipt::postcard_receipt::{PostcardReceipt, PostcardReceiptError, PostcardReceiptId};
use crate::domain::postcard_receipt::postcard_receipt_category::PostcardReceiptCategory;
use crate::domain::postcard_receipt::postcard_receipt_repository::{
  Pagination as ReceiptPagination, PostcardReceiptAddressContext, PostcardReceiptRepository,
  PostcardReceiptSearchQuery, PostcardReceiptWithContext, SortOrder as ReceiptSortOrder,
};
use crate::domain::sender::phone_number::PhoneNumber;
use crate::domain::sender::sender_entry::{SenderEntry, SenderEntryId};
use crate::domain::sender::sender_entry_repository::{
  Pagination as SenderPagination, SenderEntryRepository, SenderRepositoryError,
};
use crate::domain::sender::sender_label::SenderLabel;
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;
use crate::infrastructure::postcard_receipt::sqlx_postcard_receipt_repository::SqlxPostcardReceiptRepository;
use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

const MAX_PAGE_LIMIT: i64 = 200;
/// 連名の上限（UI と同一。API 直叩き対策でサーバー側でも検証する）
const MAX_CO_RECIPIENTS: usize = 3;
const MAX_SENDER_CO_RECIPIENTS: usize = 4;
const SENDER_DUPLICATE_LABEL_MESSAGE: &str =
  "このラベルは既に使用されています。別のラベルを指定してください。";
const RECEIPT_FUTURE_DATE_MESSAGE: &str = "受取日に未来の日付は指定できません。";
const RECEIPT_SENDER_DISPLAY_NAME_REQUIRED_MESSAGE: &str = "送り主の表示名を入力してください。";
const RECEIPT_NOT_FOUND_MESSAGE: &str = "postcard receipt not found";
const ADDRESS_ENTRY_NOT_FOUND_MESSAGE: &str = "address entry not found";
const ADDRESS_ENTRY_ARCHIVED_MESSAGE: &str = "address entry is archived";

fn map_sender_write_error(e: SenderRepositoryError, log_context: &str, fallback_code: &str) -> AppError {
  match e {
    SenderRepositoryError::DuplicateActiveLabel => {
      AppError::Validation(SENDER_DUPLICATE_LABEL_MESSAGE.to_string())
    }
    other => {
      log::error!("{} failed: {:?}", log_context, other);
      AppError::Repository(fallback_code.to_string())
    }
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      let api_logger = crate::infrastructure::logging::init_api_logger().map_err(|msg| {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg)) as Box<dyn std::error::Error>
      })?;
      app.manage(api_logger);

      // SQLite プールを初期化してアプリ全体で共有する。
      let handle = app.handle();
      let pool = tauri::async_runtime::block_on(async {
        crate::infrastructure::db::init_pool(&handle).await
      })?;
      app.manage(pool);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      create_address_entry,
      update_address_entry,
      list_address_entries,
      search_address_entries,
      archive_address_entry,
      get_address_entry,
      create_sender_entry,
      update_sender_entry,
      list_sender_entries,
      archive_sender_entry,
      get_sender_entry,
      update_sender_entry_links,
      list_sender_linked_addresses,
      get_sender_id_by_address_entry_id,
      set_sender_for_address_entry,
      get_api_log_debug_settings,
      set_api_log_debug_directory,
      set_api_log_debug_enabled,
      create_postcard_receipt,
      update_postcard_receipt,
      get_postcard_receipt,
      search_postcard_receipts,
      delete_postcard_receipt,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn get_api_log_debug_settings(logger: State<'_, Arc<ApiLogger>>) -> ApiLogDebugSettingsDto {
  logger.get_settings()
}

#[tauri::command]
fn set_api_log_debug_directory(
  logger: State<'_, Arc<ApiLogger>>,
  directory: Option<String>,
) -> Result<(), String> {
  logger.set_debug_directory(directory)
}

#[tauri::command]
fn set_api_log_debug_enabled(logger: State<'_, Arc<ApiLogger>>, enabled: bool) -> Result<(), String> {
  logger.set_debug_enabled(enabled)
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("validation error: {0}")]
  Validation(String),
  #[error("repository error: {0}")]
  Repository(String),
}

impl From<AppError> for String {
  fn from(err: AppError) -> Self {
    match err {
      AppError::Validation(msg) => msg,
      // クライアントには固定コードのみ返却し、内部詳細はログに限定する
      AppError::Repository(code) => code,
    }
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PersonNameDto {
  pub last: String,
  pub first: String,
  pub kana_last: Option<String>,
  pub kana_first: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressDto {
  pub prefecture: String,
  pub city: String,
  pub street: String,
  pub building: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressEntryDtoInput {
  pub primary_name: PersonNameDto,
  pub co_recipients: Vec<PersonNameDto>,
  pub honorific: String,
  pub postal_code: String,
  pub address: AddressDto,
  pub memo: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressEntryDto {
  pub id: String,
  pub primary_name: PersonNameDto,
  pub co_recipients: Vec<PersonNameDto>,
  pub honorific: String,
  pub postal_code: String,
  pub address: AddressDto,
  pub memo: Option<String>,
  pub archived: bool,
  pub created_at: String,
  pub updated_at: String,
}

/// 検索 API の戻り値（ページング用に total を含む）。
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressEntrySearchResult {
  pub items: Vec<AddressEntryDto>,
  pub total: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SenderEntryDtoInput {
  pub label: String,
  pub primary_name: PersonNameDto,
  pub co_recipients: Vec<PersonNameDto>,
  pub postal_code: String,
  pub address: AddressDto,
  pub phone_number: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SenderEntryDto {
  pub id: String,
  pub label: String,
  pub primary_name: PersonNameDto,
  pub co_recipients: Vec<PersonNameDto>,
  pub postal_code: String,
  pub address: AddressDto,
  pub phone_number: Option<String>,
  pub archived: bool,
  pub created_at: String,
  pub updated_at: String,
}

impl TryFrom<AddressEntryDtoInput> for AddressEntry {
  type Error = AppError;

  fn try_from(value: AddressEntryDtoInput) -> Result<Self, Self::Error> {
    if value.co_recipients.len() > MAX_CO_RECIPIENTS {
      return Err(AppError::Validation(format!(
        "連名は{}件までです（{}件指定されています）",
        MAX_CO_RECIPIENTS,
        value.co_recipients.len()
      )));
    }

    let primary = PersonName::new(
      value.primary_name.last,
      value.primary_name.first,
      value.primary_name.kana_last,
      value.primary_name.kana_first,
    )
    .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut co_recipients = Vec::with_capacity(value.co_recipients.len());
    for c in value.co_recipients {
      let name = PersonName::new(c.last, c.first, c.kana_last, c.kana_first)
        .map_err(|e| AppError::Validation(e.to_string()))?;
      co_recipients.push(name);
    }

    let honorific =
      Honorific::from_str(&value.honorific).map_err(|e| AppError::Validation(e.to_string()))?;
    let postal =
      PostalCode::new(value.postal_code).map_err(|e| AppError::Validation(e.to_string()))?;
    let addr = Address::new(
      value.address.prefecture,
      value.address.city,
      value.address.street,
      value.address.building,
    )
    .map_err(|e| AppError::Validation(e.to_string()))?;
    let memo = match value.memo {
      Some(text) => Some(
        Memo::new(text).map_err(|e| AppError::Validation(e.to_string()))?,
      ),
      None => None,
    };

    Ok(AddressEntry::create_new(
      primary,
      co_recipients,
      honorific,
      postal,
      addr,
      memo,
    ))
  }
}

impl From<AddressEntry> for AddressEntryDto {
  fn from(entry: AddressEntry) -> Self {
    let primary = entry.primary_name().clone();
    let co = entry.co_recipients().to_vec();
    let addr = entry.address().clone();
    let memo = entry.memo().map(|m| m.text().to_string());

    AddressEntryDto {
      id: entry.id().as_uuid().to_string(),
      primary_name: PersonNameDto {
        last: primary.last().to_string(),
        first: primary.first().to_string(),
        kana_last: primary.kana_last().map(|s| s.to_string()),
        kana_first: primary.kana_first().map(|s| s.to_string()),
      },
      co_recipients: co
        .iter()
        .map(|c| PersonNameDto {
          last: c.last().to_string(),
          first: c.first().to_string(),
          kana_last: c.kana_last().map(|s| s.to_string()),
          kana_first: c.kana_first().map(|s| s.to_string()),
        })
        .collect(),
      honorific: entry.honorific().as_str().to_string(),
      postal_code: entry.postal_code().value().to_string(),
      address: AddressDto {
        prefecture: addr.prefecture().to_string(),
        city: addr.city().to_string(),
        street: addr.street().to_string(),
        building: addr.building().map(|s| s.to_string()),
      },
      memo,
      archived: entry.archived(),
      created_at: entry.created_at().to_rfc3339(),
      updated_at: entry.updated_at().to_rfc3339(),
    }
  }
}

impl TryFrom<SenderEntryDtoInput> for SenderEntry {
  type Error = AppError;

  fn try_from(value: SenderEntryDtoInput) -> Result<Self, Self::Error> {
    if value.co_recipients.len() > MAX_SENDER_CO_RECIPIENTS {
      return Err(AppError::Validation(format!(
        "連名は{}件までです（{}件指定されています）",
        MAX_SENDER_CO_RECIPIENTS,
        value.co_recipients.len()
      )));
    }

    let label = SenderLabel::new(value.label).map_err(|e| AppError::Validation(e.to_string()))?;
    let primary = PersonName::new(
      value.primary_name.last,
      value.primary_name.first,
      value.primary_name.kana_last,
      value.primary_name.kana_first,
    )
    .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut co_recipients = Vec::with_capacity(value.co_recipients.len());
    for c in value.co_recipients {
      let name = PersonName::new(c.last, c.first, c.kana_last, c.kana_first)
        .map_err(|e| AppError::Validation(e.to_string()))?;
      co_recipients.push(name);
    }

    let postal =
      PostalCode::new(value.postal_code).map_err(|e| AppError::Validation(e.to_string()))?;
    let addr = Address::new(
      value.address.prefecture,
      value.address.city,
      value.address.street,
      value.address.building,
    )
    .map_err(|e| AppError::Validation(e.to_string()))?;

    let phone_number = match value.phone_number {
      Some(text) => Some(
        PhoneNumber::new(text).map_err(|e| AppError::Validation(e.to_string()))?,
      ),
      None => None,
    };

    SenderEntry::create_new(label, primary, co_recipients, postal, addr, phone_number)
      .map_err(|e| AppError::Validation(e.to_string()))
  }
}

impl From<SenderEntry> for SenderEntryDto {
  fn from(entry: SenderEntry) -> Self {
    let primary = entry.primary_name().clone();
    let co = entry.co_recipients().to_vec();
    let addr = entry.address().clone();
    let phone_number = entry.phone_number().map(|p| p.value().to_string());
    SenderEntryDto {
      id: entry.id().as_uuid().to_string(),
      label: entry.label().value().to_string(),
      primary_name: PersonNameDto {
        last: primary.last().to_string(),
        first: primary.first().to_string(),
        kana_last: primary.kana_last().map(|s| s.to_string()),
        kana_first: primary.kana_first().map(|s| s.to_string()),
      },
      co_recipients: co
        .iter()
        .map(|c| PersonNameDto {
          last: c.last().to_string(),
          first: c.first().to_string(),
          kana_last: c.kana_last().map(|s| s.to_string()),
          kana_first: c.kana_first().map(|s| s.to_string()),
        })
        .collect(),
      postal_code: entry.postal_code().value().to_string(),
      address: AddressDto {
        prefecture: addr.prefecture().to_string(),
        city: addr.city().to_string(),
        street: addr.street().to_string(),
        building: addr.building().map(|s| s.to_string()),
      },
      phone_number,
      archived: entry.archived(),
      created_at: entry.created_at().to_rfc3339(),
      updated_at: entry.updated_at().to_rfc3339(),
    }
  }
}

#[tauri::command]
async fn create_address_entry(
  pool: State<'_, SqlitePool>,
  dto: AddressEntryDtoInput,
) -> Result<(), String> {
  let entry =
    AddressEntry::try_from(dto).map_err::<String, _>(|e| e.into()).map_err(|e| e.to_string())?;
  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());

  repo
    .create(&entry)
    .await
    .map_err(|e| {
      log::error!("create_address_entry failed: {:?}", e);
      AppError::Repository("ADDR_CREATE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn update_address_entry(
  pool: State<'_, SqlitePool>,
  id: String,
  dto: AddressEntryDtoInput,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = AddressEntryId::from_uuid(uuid);
  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());

  // 既存エントリを取得し、created_at / archived_at を維持する。
  let existing = repo
    .find_by_id(&id)
    .await
    .map_err(|e| {
      log::error!("update_address_entry find_by_id failed: {:?}", e);
      String::from(AppError::Repository("ADDR_UPDATE_FAILED".to_string()))
    })?
    .ok_or_else(|| AppError::Validation("address entry not found".to_string()))?;

  let new_values =
    AddressEntry::try_from(dto).map_err::<String, _>(|e| e.into()).map_err(|e| e.to_string())?;

  let now = Utc::now();
  let entry = AddressEntry::from_persisted(
    id,
    new_values.primary_name().clone(),
    new_values.co_recipients().to_vec(),
    new_values.honorific(),
    new_values.postal_code().clone(),
    new_values.address().clone(),
    new_values.memo().cloned(),
    existing.archived_at(),
    existing.created_at(),
    now,
  );

  repo
    .update(&entry)
    .await
    .map_err(|e| {
      log::error!("update_address_entry failed: {:?}", e);
      AppError::Repository("ADDR_UPDATE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn list_address_entries(
  pool: State<'_, SqlitePool>,
  limit: i64,
  offset: i64,
) -> Result<Vec<AddressEntryDto>, String> {
  if limit < 1 || limit > MAX_PAGE_LIMIT {
    return Err(
      AppError::Validation(format!("limit must be between 1 and {}", MAX_PAGE_LIMIT))
        .to_string(),
    );
  }
  if offset < 0 {
    return Err(AppError::Validation("offset must be >= 0".to_string()).to_string());
  }

  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let entries = repo
    .list_active(Pagination { limit, offset })
    .await
    .map_err(|e| {
      log::error!("list_address_entries failed: {:?}", e);
      AppError::Repository("ADDR_LIST_FAILED".to_string())
    })?;

  Ok(entries.into_iter().map(AddressEntryDto::from).collect())
}

const DEFAULT_SEARCH_LIMIT: i64 = 50;
const DEFAULT_SEARCH_OFFSET: i64 = 0;

#[tauri::command]
async fn search_address_entries(
  pool: State<'_, SqlitePool>,
  keyword: Option<String>,
  sort_key: String,
  sort_order: String,
  include_archived: bool,
  limit: Option<i64>,
  offset: Option<i64>,
) -> Result<AddressEntrySearchResult, String> {
  let sort_key = match sort_key.as_str() {
    "updated_at" => SortKey::UpdatedAt,
    _ => SortKey::NameKana,
  };
  let sort_order = match sort_order.as_str() {
    "desc" => SortOrder::Desc,
    _ => SortOrder::Asc,
  };

  // limit/offset が未指定の場合はデフォルトを補完（全件取得を防ぐ）。
  let (l, o) = (limit.unwrap_or(DEFAULT_SEARCH_LIMIT), offset.unwrap_or(DEFAULT_SEARCH_OFFSET));
  if l < 1 || l > MAX_PAGE_LIMIT {
    return Err(
      AppError::Validation(format!("limit must be between 1 and {}", MAX_PAGE_LIMIT)).to_string(),
    );
  }
  if o < 0 {
    return Err(AppError::Validation("offset must be >= 0".to_string()).to_string());
  }
  let pagination = Pagination { limit: l, offset: o };

  let query = AddressSearchQuery {
    keyword,
    sort_key,
    sort_order,
    include_archived,
    pagination: Some(pagination),
  };

  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let (entries, total) = repo
    .search(query)
    .await
    .map_err(|e| {
      log::error!("search_address_entries failed: {:?}", e);
      String::from(AppError::Repository("ADDR_SEARCH_FAILED".to_string()))
    })?;

  Ok(AddressEntrySearchResult {
    items: entries.into_iter().map(AddressEntryDto::from).collect(),
    total,
  })
}

#[tauri::command]
async fn archive_address_entry(
  pool: State<'_, SqlitePool>,
  id: String,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = AddressEntryId::from_uuid(uuid);
  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());

  repo
    .archive(&id)
    .await
    .map_err(|e| {
      log::error!("archive_address_entry failed: {:?}", e);
      AppError::Repository("ADDR_ARCHIVE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn get_address_entry(
  pool: State<'_, SqlitePool>,
  id: String,
) -> Result<AddressEntryDto, String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = AddressEntryId::from_uuid(uuid);
  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());

  let entry = repo
    .find_by_id(&id)
    .await
    .map_err(|e| {
      log::error!("get_address_entry failed: {:?}", e);
      AppError::Repository("ADDR_GET_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation("address entry not found".to_string()).to_string())?;

  Ok(AddressEntryDto::from(entry))
}

#[tauri::command]
async fn create_sender_entry(
  pool: State<'_, SqlitePool>,
  dto: SenderEntryDtoInput,
) -> Result<(), String> {
  create_sender_entry_impl(pool.inner(), dto).await
}

async fn create_sender_entry_impl(pool: &SqlitePool, dto: SenderEntryDtoInput) -> Result<(), String> {
  let entry =
    SenderEntry::try_from(dto).map_err::<String, _>(|e| e.into()).map_err(|e| e.to_string())?;
  let repo = SqlxSenderEntryRepository::new(pool.clone());
  let duplicated = repo
    .exists_active_label(entry.label().value(), None)
    .await
    .map_err(|e| {
      log::error!("create_sender_entry exists_active_label failed: {:?}", e);
      AppError::Repository("SENDER_CREATE_FAILED".to_string())
    })?;
  if duplicated {
    return Err(String::from(AppError::Validation(
      SENDER_DUPLICATE_LABEL_MESSAGE.to_string(),
    )));
  }
  repo
    .create(&entry)
    .await
    .map_err(|e| map_sender_write_error(e, "create_sender_entry", "SENDER_CREATE_FAILED"))?;
  Ok(())
}

#[tauri::command]
async fn update_sender_entry(
  pool: State<'_, SqlitePool>,
  id: String,
  dto: SenderEntryDtoInput,
) -> Result<(), String> {
  update_sender_entry_impl(pool.inner(), id, dto).await
}

async fn update_sender_entry_impl(
  pool: &SqlitePool,
  id: String,
  dto: SenderEntryDtoInput,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = SenderEntryId::from_uuid(uuid);
  let repo = SqlxSenderEntryRepository::new(pool.clone());

  let existing = repo
    .find_by_id(&id)
    .await
    .map_err(|e| {
      log::error!("update_sender_entry find_by_id failed: {:?}", e);
      String::from(AppError::Repository("SENDER_UPDATE_FAILED".to_string()))
    })?
    .ok_or_else(|| AppError::Validation("sender entry not found".to_string()))?;

  let new_values =
    SenderEntry::try_from(dto).map_err::<String, _>(|e| e.into()).map_err(|e| e.to_string())?;
  let duplicated = repo
    .exists_active_label(new_values.label().value(), Some(&id))
    .await
    .map_err(|e| {
      log::error!("update_sender_entry exists_active_label failed: {:?}", e);
      AppError::Repository("SENDER_UPDATE_FAILED".to_string())
    })?;
  if duplicated {
    return Err(String::from(AppError::Validation(
      SENDER_DUPLICATE_LABEL_MESSAGE.to_string(),
    )));
  }
  let now = Utc::now();
  let entry = SenderEntry::from_persisted(
    id,
    new_values.label().clone(),
    new_values.primary_name().clone(),
    new_values.co_recipients().to_vec(),
    new_values.postal_code().clone(),
    new_values.address().clone(),
    new_values.phone_number().cloned(),
    existing.archived_at(),
    existing.created_at(),
    now,
  )
  .map_err(|e| AppError::Validation(e.to_string()).to_string())?;

  repo
    .update(&entry)
    .await
    .map_err(|e| map_sender_write_error(e, "update_sender_entry", "SENDER_UPDATE_FAILED"))?;
  Ok(())
}

#[tauri::command]
async fn list_sender_entries(
  pool: State<'_, SqlitePool>,
  limit: i64,
  offset: i64,
) -> Result<Vec<SenderEntryDto>, String> {
  if limit < 1 || limit > MAX_PAGE_LIMIT {
    return Err(
      AppError::Validation(format!("limit must be between 1 and {}", MAX_PAGE_LIMIT))
        .to_string(),
    );
  }
  if offset < 0 {
    return Err(AppError::Validation("offset must be >= 0".to_string()).to_string());
  }

  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  let entries = repo
    .list_active(SenderPagination { limit, offset })
    .await
    .map_err(|e| {
      log::error!("list_sender_entries failed: {:?}", e);
      AppError::Repository("SENDER_LIST_FAILED".to_string())
    })?;

  Ok(entries.into_iter().map(SenderEntryDto::from).collect())
}

#[tauri::command]
async fn archive_sender_entry(
  pool: State<'_, SqlitePool>,
  id: String,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = SenderEntryId::from_uuid(uuid);
  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  repo
    .archive(&id)
    .await
    .map_err(|e| {
      log::error!("archive_sender_entry failed: {:?}", e);
      AppError::Repository("SENDER_ARCHIVE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn get_sender_entry(
  pool: State<'_, SqlitePool>,
  id: String,
) -> Result<SenderEntryDto, String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = SenderEntryId::from_uuid(uuid);
  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  let entry = repo
    .find_by_id(&id)
    .await
    .map_err(|e| {
      log::error!("get_sender_entry failed: {:?}", e);
      AppError::Repository("SENDER_GET_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation("sender entry not found".to_string()).to_string())?;
  Ok(SenderEntryDto::from(entry))
}

#[tauri::command]
async fn update_sender_entry_links(
  pool: State<'_, SqlitePool>,
  sender_id: String,
  address_entry_ids: Vec<String>,
) -> Result<(), String> {
  update_sender_entry_links_impl(pool.inner(), sender_id, address_entry_ids).await
}

async fn update_sender_entry_links_impl(
  pool: &SqlitePool,
  sender_id: String,
  address_entry_ids: Vec<String>,
) -> Result<(), String> {
  let sender_uuid =
    Uuid::parse_str(&sender_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = SenderEntryId::from_uuid(sender_uuid);

  let mut parsed_address_ids = Vec::with_capacity(address_entry_ids.len());
  for id in address_entry_ids {
    let parsed =
      Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
    parsed_address_ids.push(parsed);
  }

  let repo = SqlxSenderEntryRepository::new(pool.clone());
  // sender の実在 + 未アーカイブを検証
  let sender = repo
    .find_by_id(&sender_entry_id)
    .await
    .map_err(|e| {
      log::error!("update_sender_entry_links find_by_id failed: {:?}", e);
      AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation("sender entry not found".to_string()).to_string())?;
  if sender.archived() {
    return Err(AppError::Validation("sender entry is archived".to_string()).to_string());
  }

  // address_entries の実在 + 未アーカイブを検証（0件は解除扱いなのでOK）
  if !parsed_address_ids.is_empty() {
    validate_active_address_entries(pool, &parsed_address_ids).await?;
  }

  repo
    .replace_links_for_sender(&sender_entry_id, &parsed_address_ids)
    .await
    .map_err(|e| {
      log::error!("update_sender_entry_links failed: {:?}", e);
      AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn list_sender_linked_addresses(
  pool: State<'_, SqlitePool>,
  sender_id: String,
) -> Result<Vec<AddressEntryDto>, String> {
  list_sender_linked_addresses_impl(pool.inner(), sender_id).await
}

async fn list_sender_linked_addresses_impl(
  pool: &SqlitePool,
  sender_id: String,
) -> Result<Vec<AddressEntryDto>, String> {
  let sender_uuid =
    Uuid::parse_str(&sender_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = SenderEntryId::from_uuid(sender_uuid);

  let repo = SqlxSenderEntryRepository::new(pool.clone());
  let sender = repo
    .find_by_id(&sender_entry_id)
    .await
    .map_err(|e| {
      log::error!("list_sender_linked_addresses find_by_id failed: {:?}", e);
      AppError::Repository("SENDER_LINK_LIST_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation("sender entry not found".to_string()).to_string())?;
  if sender.archived() {
    return Err(AppError::Validation("sender entry is archived".to_string()).to_string());
  }

  let entries = repo
    .list_linked_address_entries(&sender_entry_id)
    .await
    .map_err(|e| {
      log::error!("list_sender_linked_addresses failed: {:?}", e);
      AppError::Repository("SENDER_LINK_LIST_FAILED".to_string())
    })?;

  Ok(entries.into_iter().map(AddressEntryDto::from).collect())
}

#[tauri::command]
async fn get_sender_id_by_address_entry_id(
  pool: State<'_, SqlitePool>,
  address_entry_id: String,
) -> Result<Option<String>, String> {
  let address_uuid =
    Uuid::parse_str(&address_entry_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  let sender_id = repo
    .find_sender_id_by_address_entry_id(address_uuid)
    .await
    .map_err(|e| {
      log::error!("get_sender_id_by_address_entry_id failed: {:?}", e);
      AppError::Repository("SENDER_LINK_LOOKUP_FAILED".to_string())
    })?
    .map(|id| id.as_uuid().to_string());
  Ok(sender_id)
}

#[tauri::command]
async fn set_sender_for_address_entry(
  pool: State<'_, SqlitePool>,
  address_entry_id: String,
  sender_id: Option<String>,
) -> Result<(), String> {
  set_sender_for_address_entry_impl(pool.inner(), address_entry_id, sender_id).await
}

async fn set_sender_for_address_entry_impl(
  pool: &SqlitePool,
  address_entry_id: String,
  sender_id: Option<String>,
) -> Result<(), String> {
  let address_uuid =
    Uuid::parse_str(&address_entry_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  // address の実在 + 未アーカイブを検証
  let address_repo = SqlxAddressEntryRepository::new(pool.clone());
  let addr_id = AddressEntryId::from_uuid(address_uuid);
  let addr = address_repo
    .find_by_id(&addr_id)
    .await
    .map_err(|e| {
      log::error!("set_sender_for_address_entry find address failed: {:?}", e);
      AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation("address entry not found".to_string()).to_string())?;
  if addr.archived() {
    return Err(AppError::Validation("address entry is archived".to_string()).to_string());
  }

  let sender_entry_id = match sender_id {
    Some(s) => {
      let uuid =
        Uuid::parse_str(&s).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
      Some(SenderEntryId::from_uuid(uuid))
    }
    None => None,
  };

  let repo = SqlxSenderEntryRepository::new(pool.clone());
  // sender の実在 + 未アーカイブを検証（Some の場合）
  if let Some(ref sid) = sender_entry_id {
    let sender = repo
      .find_by_id(sid)
      .await
      .map_err(|e| {
        log::error!("set_sender_for_address_entry find sender failed: {:?}", e);
        AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
      })?
      .ok_or_else(|| AppError::Validation("sender entry not found".to_string()).to_string())?;
    if sender.archived() {
      return Err(AppError::Validation("sender entry is archived".to_string()).to_string());
    }
  }

  repo
    .set_sender_for_address(address_uuid, sender_entry_id.as_ref())
    .await
    .map_err(|e| {
      log::error!("set_sender_for_address_entry failed: {:?}", e);
      AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
    })?;

  Ok(())
}

async fn validate_active_address_entries(
  pool: &SqlitePool,
  address_ids: &[Uuid],
) -> Result<(), String> {
  // 重複を除外
  let mut unique: Vec<String> = address_ids.iter().map(|u| u.to_string()).collect();
  unique.sort();
  unique.dedup();

  const IN_CHUNK_SIZE: usize = 100;
  let mut found = 0i64;

  for chunk in unique.chunks(IN_CHUNK_SIZE) {
    let placeholders = chunk
      .iter()
      .enumerate()
      .map(|(i, _)| format!("?{}", i + 1))
      .collect::<Vec<_>>()
      .join(",");
    let sql = format!(
      "SELECT COUNT(*) AS cnt FROM address_entries WHERE archived_at IS NULL AND id IN ({})",
      placeholders
    );
    let mut q = sqlx::query(&sql);
    for id in chunk {
      q = q.bind(id);
    }
    let cnt: i64 = q
      .fetch_one(pool)
      .await
      .map_err(|e| {
        log::error!("validate_active_address_entries failed: {:?}", e);
        AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
      })?
      .get("cnt");
    found += cnt;
  }

  if found != unique.len() as i64 {
    return Err(AppError::Validation("address entry not found".to_string()).to_string());
  }
  Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostcardReceiptDtoInput {
  pub address_entry_id: Option<String>,
  pub sender_display_name: Option<String>,
  pub received_at: String,
  pub category: String,
  pub memo: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostcardReceiptDto {
  pub id: String,
  pub address_entry_id: Option<String>,
  pub sender_display_name: Option<String>,
  pub received_at: String,
  pub category: String,
  pub memo: Option<String>,
  pub created_at: String,
  pub updated_at: String,
  pub address_entry_display_name: Option<String>,
  pub address_entry_address_line: Option<String>,
  pub address_entry_archived: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostcardReceiptSearchResult {
  pub items: Vec<PostcardReceiptDto>,
  pub total: i64,
}

fn map_postcard_receipt_error(err: PostcardReceiptError) -> AppError {
  match err {
    PostcardReceiptError::FutureReceivedDate => {
      AppError::Validation(RECEIPT_FUTURE_DATE_MESSAGE.to_string())
    }
    PostcardReceiptError::SenderDisplayNameRequired => {
      AppError::Validation(RECEIPT_SENDER_DISPLAY_NAME_REQUIRED_MESSAGE.to_string())
    }
    PostcardReceiptError::InvalidCategory(e) => AppError::Validation(e.to_string()),
    PostcardReceiptError::InvalidMemo(e) => AppError::Validation(e.to_string()),
  }
}

fn postcard_receipt_dto_from_context(ctx: PostcardReceiptWithContext) -> PostcardReceiptDto {
  let receipt = ctx.receipt;
  let (address_entry_display_name, address_entry_address_line, address_entry_archived) =
    match ctx.address {
      Some(PostcardReceiptAddressContext {
        display_name,
        address_line,
        archived,
      }) => (Some(display_name), Some(address_line), Some(archived)),
      None => (None, None, None),
    };

  PostcardReceiptDto {
    id: receipt.id().as_uuid().to_string(),
    address_entry_id: receipt.address_entry_id().map(|u| u.to_string()),
    sender_display_name: receipt.sender_display_name().map(str::to_string),
    received_at: receipt.received_at().format("%Y-%m-%d").to_string(),
    category: receipt.category().as_str().to_string(),
    memo: receipt.memo().map(|m| m.text().to_string()),
    created_at: receipt.created_at().to_rfc3339(),
    updated_at: receipt.updated_at().to_rfc3339(),
    address_entry_display_name,
    address_entry_address_line,
    address_entry_archived,
  }
}

/// 受取履歴の住所録紐付け検証。
/// - create / 新しい ID への差し替え: active のみ許可
/// - update で既存と同じ ID: archived でも許可（履歴の継続編集）
async fn validate_address_entry_for_receipt(
  pool: &SqlitePool,
  address_entry_id: &Uuid,
  allow_archived_if_same_as: Option<Uuid>,
) -> Result<(), String> {
  let repo = SqlxAddressEntryRepository::new(pool.clone());
  let entry_id = AddressEntryId::from_uuid(*address_entry_id);
  let found = repo
    .find_by_id(&entry_id)
    .await
    .map_err(|e| {
      log::error!("validate_address_entry_for_receipt failed: {:?}", e);
      AppError::Repository("RECEIPT_ADDRESS_LOOKUP_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation(ADDRESS_ENTRY_NOT_FOUND_MESSAGE.to_string()).to_string())?;

  if found.archived() {
    let keep_existing_link = allow_archived_if_same_as == Some(*address_entry_id);
    if !keep_existing_link {
      return Err(AppError::Validation(ADDRESS_ENTRY_ARCHIVED_MESSAGE.to_string()).to_string());
    }
  }
  Ok(())
}

async fn build_postcard_receipt_values_from_input(
  pool: &SqlitePool,
  dto: PostcardReceiptDtoInput,
  allow_archived_if_same_as: Option<Uuid>,
) -> Result<
  (
    Option<Uuid>,
    Option<String>,
    chrono::NaiveDate,
    PostcardReceiptCategory,
    Option<crate::domain::address::memo::Memo>,
  ),
  String,
> {
  let address_entry_id = match dto.address_entry_id {
    Some(id) => {
      let uuid =
        Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
      validate_address_entry_for_receipt(pool, &uuid, allow_archived_if_same_as).await?;
      Some(uuid)
    }
    None => None,
  };

  let received_at = chrono::NaiveDate::parse_from_str(&dto.received_at, "%Y-%m-%d")
    .map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let category = PostcardReceiptCategory::parse(&dto.category)
    .map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let memo = match dto.memo {
    Some(text) if !text.is_empty() => Some(
      crate::domain::address::memo::Memo::new(text)
        .map_err(|e| AppError::Validation(e.to_string()).to_string())?,
    ),
    _ => None,
  };

  Ok((
    address_entry_id,
    dto.sender_display_name,
    received_at,
    category,
    memo,
  ))
}

#[tauri::command]
async fn create_postcard_receipt(
  pool: State<'_, SqlitePool>,
  dto: PostcardReceiptDtoInput,
) -> Result<String, String> {
  create_postcard_receipt_impl(pool.inner(), dto).await
}

async fn create_postcard_receipt_impl(
  pool: &SqlitePool,
  dto: PostcardReceiptDtoInput,
) -> Result<String, String> {
  let (address_entry_id, sender_display_name, received_at, category, memo) =
    build_postcard_receipt_values_from_input(pool, dto, None).await?;

  let receipt = PostcardReceipt::create_new(
    address_entry_id,
    sender_display_name,
    received_at,
    category,
    memo,
  )
  .map_err(|e| map_postcard_receipt_error(e).to_string())?;

  let id = receipt.id().as_uuid().to_string();
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());
  repo
    .create(&receipt)
    .await
    .map_err(|e| {
      log::error!("create_postcard_receipt failed: {:?}", e);
      AppError::Repository("RECEIPT_CREATE_FAILED".to_string())
    })?;
  Ok(id)
}

#[tauri::command]
async fn update_postcard_receipt(
  pool: State<'_, SqlitePool>,
  id: String,
  dto: PostcardReceiptDtoInput,
) -> Result<(), String> {
  update_postcard_receipt_impl(pool.inner(), id, dto).await
}

async fn update_postcard_receipt_impl(
  pool: &SqlitePool,
  id: String,
  dto: PostcardReceiptDtoInput,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let receipt_id = PostcardReceiptId::from_uuid(uuid);
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());

  let existing = repo
    .find_by_id(&receipt_id)
    .await
    .map_err(|e| {
      log::error!("update_postcard_receipt find_by_id failed: {:?}", e);
      AppError::Repository("RECEIPT_UPDATE_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()))?;

  if existing.receipt.is_deleted() {
    return Err(AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()).to_string());
  }

  let (address_entry_id, sender_display_name, received_at, category, memo) =
    build_postcard_receipt_values_from_input(
      pool,
      dto,
      existing.receipt.address_entry_id(),
    )
    .await?;

  let receipt = PostcardReceipt::from_persisted(
    receipt_id,
    address_entry_id,
    sender_display_name,
    received_at,
    category,
    memo,
    existing.receipt.deleted_at(),
    existing.receipt.created_at(),
    Utc::now(),
  )
  .map_err(|e| map_postcard_receipt_error(e).to_string())?;

  repo
    .update(&receipt)
    .await
    .map_err(|e| match e {
      crate::domain::postcard_receipt::postcard_receipt_repository::PostcardReceiptRepositoryError::NotFound => {
        AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()).to_string()
      }
      other => {
        log::error!("update_postcard_receipt failed: {:?}", other);
        AppError::Repository("RECEIPT_UPDATE_FAILED".to_string()).to_string()
      }
    })?;
  Ok(())
}

#[tauri::command]
async fn get_postcard_receipt(
  pool: State<'_, SqlitePool>,
  id: String,
) -> Result<PostcardReceiptDto, String> {
  get_postcard_receipt_impl(pool.inner(), id).await
}

async fn get_postcard_receipt_impl(pool: &SqlitePool, id: String) -> Result<PostcardReceiptDto, String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let receipt_id = PostcardReceiptId::from_uuid(uuid);
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());

  let found = repo
    .find_by_id(&receipt_id)
    .await
    .map_err(|e| {
      log::error!("get_postcard_receipt failed: {:?}", e);
      AppError::Repository("RECEIPT_GET_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()))?;

  if found.receipt.is_deleted() {
    return Err(AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()).to_string());
  }

  Ok(postcard_receipt_dto_from_context(found))
}

#[tauri::command]
async fn search_postcard_receipts(
  pool: State<'_, SqlitePool>,
  keyword: Option<String>,
  year: Option<i32>,
  category: Option<String>,
  address_entry_id: Option<String>,
  include_deleted: Option<bool>,
  limit: Option<i64>,
  offset: Option<i64>,
  sort_order: Option<String>,
) -> Result<PostcardReceiptSearchResult, String> {
  let (l, o) = (
    limit.unwrap_or(DEFAULT_SEARCH_LIMIT),
    offset.unwrap_or(DEFAULT_SEARCH_OFFSET),
  );
  if l < 1 || l > MAX_PAGE_LIMIT {
    return Err(
      AppError::Validation(format!("limit must be between 1 and {}", MAX_PAGE_LIMIT)).to_string(),
    );
  }
  if o < 0 {
    return Err(AppError::Validation("offset must be >= 0".to_string()).to_string());
  }

  let parsed_category = match category {
    Some(value) if !value.is_empty() => Some(
      PostcardReceiptCategory::parse(&value)
        .map_err(|e| AppError::Validation(e.to_string()).to_string())?,
    ),
    _ => None,
  };

  let parsed_address_entry_id = match address_entry_id {
    Some(id) if !id.is_empty() => Some(
      Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?,
    ),
    _ => None,
  };

  let sort_order = match sort_order.as_deref() {
    Some("asc") => ReceiptSortOrder::Asc,
    _ => ReceiptSortOrder::Desc,
  };

  let query = PostcardReceiptSearchQuery {
    keyword: keyword.filter(|k| !k.trim().is_empty()),
    year,
    category: parsed_category,
    address_entry_id: parsed_address_entry_id,
    include_deleted: include_deleted.unwrap_or(false),
    pagination: ReceiptPagination { limit: l, offset: o },
    sort_order,
  };

  let repo = SqlxPostcardReceiptRepository::new(pool.inner().clone());
  let (items, total) = repo.search(query).await.map_err(|e| {
    log::error!("search_postcard_receipts failed: {:?}", e);
    AppError::Repository("RECEIPT_SEARCH_FAILED".to_string())
  })?;

  Ok(PostcardReceiptSearchResult {
    items: items
      .into_iter()
      .map(postcard_receipt_dto_from_context)
      .collect(),
    total,
  })
}

#[tauri::command]
async fn delete_postcard_receipt(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
  delete_postcard_receipt_impl(pool.inner(), id).await
}

async fn delete_postcard_receipt_impl(pool: &SqlitePool, id: String) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let receipt_id = PostcardReceiptId::from_uuid(uuid);
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());
  repo
    .delete(&receipt_id)
    .await
    .map_err(|e| match e {
      crate::domain::postcard_receipt::postcard_receipt_repository::PostcardReceiptRepositoryError::NotFound => {
        AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string())
      }
      other => {
        log::error!("delete_postcard_receipt failed: {:?}", other);
        AppError::Repository("RECEIPT_DELETE_FAILED".to_string())
      }
    })?;
  Ok(())
}
