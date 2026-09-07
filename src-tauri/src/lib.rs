mod domain;
mod infrastructure;
#[cfg(test)]
mod command_tests;

use std::sync::Arc;

use chrono::{DateTime, Utc};
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
use crate::domain::print::postcard_send::PostcardSend;
use crate::domain::print::postcard_send_repository::{
  PostcardSendRepository, PostcardSendRepositoryError,
};
use crate::domain::print::postcard_type::PostcardType;
use crate::domain::print::print_layout_preference::PrintLayoutPreference;
use crate::domain::print::print_layout_preference_repository::PrintLayoutPreferenceRepository;
use crate::domain::print::print_snapshot::{
  AddressPrintSnapshot, PrintSnapshotError, SenderPrintSnapshot,
};
use crate::domain::sender::phone_number::PhoneNumber;
use crate::domain::sender::sender_entry::{SenderEntry, SenderEntryId};
use crate::domain::sender::sender_entry_repository::{
  Pagination as SenderPagination, SenderEntryRepository, SenderRepositoryError,
};
use crate::domain::sender::sender_label::SenderLabel;
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;
use crate::infrastructure::postcard_receipt::sqlx_postcard_receipt_repository::SqlxPostcardReceiptRepository;
use crate::infrastructure::print::sqlx_postcard_send_repository::SqlxPostcardSendRepository;
use crate::infrastructure::print::sqlx_print_layout_preference_repository::SqlxPrintLayoutPreferenceRepository;
use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

const MAX_PAGE_LIMIT: i64 = 200;
/// 印刷ジョブの宛名選択上限（設計 FR-01）
const MAX_PRINT_ADDRESS_ENTRY_IDS: usize = 200;
/// レイアウト prefs の layer_id allowlist（フロント ALL_PRINT_LAYER_IDS と同期）
const PRINT_LAYER_ID_ALLOWLIST: &[&str] = &[
  "recipient.postalCode",
  "recipient.address1",
  "recipient.address2",
  "recipient.address3",
  "recipient.primaryLast",
  "recipient.primaryFirst",
  "recipient.honorific",
  "recipient.coLast.1",
  "recipient.coFirst.1",
  "recipient.coHonorific.1",
  "recipient.coLast.2",
  "recipient.coFirst.2",
  "recipient.coHonorific.2",
  "recipient.coLast.3",
  "recipient.coFirst.3",
  "recipient.coHonorific.3",
  "sender.postalCode",
  "sender.address1",
  "sender.address2",
  "sender.address3",
  "sender.primaryLast",
  "sender.primaryFirst",
  "sender.coLast.1",
  "sender.coFirst.1",
  "sender.coLast.2",
  "sender.coFirst.2",
  "sender.coLast.3",
  "sender.coFirst.3",
  "sender.coLast.4",
  "sender.coFirst.4",
];
/// 連名の上限（UI と同一。API 直叩き対策でサーバー側でも検証する）
const MAX_CO_RECIPIENTS: usize = 3;
const MAX_SENDER_CO_RECIPIENTS: usize = 4;
const SENDER_DUPLICATE_LABEL_MESSAGE: &str =
  "このラベルは既に使用されています。別のラベルを指定してください。";
const RECEIPT_FUTURE_DATE_MESSAGE: &str = "受取日に未来の日付は指定できません。";
const RECEIPT_SENDER_DISPLAY_NAME_REQUIRED_MESSAGE: &str = "送り主の表示名を入力してください。";
const RECEIPT_NOT_FOUND_MESSAGE: &str = "postcard receipt not found";
const RECEIPT_CONFLICT_MESSAGE: &str =
  "他の操作で更新済みです。画面を再読み込みしてから再度保存してください。";
const ADDRESS_ENTRY_NOT_FOUND_MESSAGE: &str = "address entry not found";
const ADDRESS_ENTRY_ARCHIVED_MESSAGE: &str = "address entry is archived";
const SENDER_ENTRY_NOT_FOUND_MESSAGE: &str = "sender entry not found";
const SENDER_ENTRY_ARCHIVED_MESSAGE: &str = "sender entry is archived";

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
      list_postcard_receipt_years,
      delete_postcard_receipt,
      filter_active_address_entry_ids,
      resolve_print_job_items,
      build_address_print_snapshot,
      build_sender_print_snapshot,
      list_print_layout_preferences,
      save_print_layout_preferences,
      create_postcard_sends_batch,
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

/// postcard command の rejection 文字列（`Display` ではなく `From<AppError>` と揃える）
fn postcard_command_error(err: AppError) -> String {
  String::from(err)
}

fn map_postcard_receipt_write_error(
  err: crate::domain::postcard_receipt::postcard_receipt_repository::PostcardReceiptRepositoryError,
  log_context: &str,
  fallback_code: &str,
) -> String {
  use crate::domain::postcard_receipt::postcard_receipt_repository::PostcardReceiptRepositoryError;
  match err {
    PostcardReceiptRepositoryError::NotFound => {
      postcard_command_error(AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()))
    }
    PostcardReceiptRepositoryError::AddressLinkRejected => {
      // 事前検証後の競合: 多くの場合 archive への並行変更
      postcard_command_error(AppError::Validation(ADDRESS_ENTRY_ARCHIVED_MESSAGE.to_string()))
    }
    PostcardReceiptRepositoryError::Conflict => {
      postcard_command_error(AppError::Validation(RECEIPT_CONFLICT_MESSAGE.to_string()))
    }
    other => {
      log::error!("{log_context} failed: {:?}", other);
      postcard_command_error(AppError::Repository(fallback_code.to_string()))
    }
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
    .ok_or_else(|| postcard_command_error(AppError::Validation(ADDRESS_ENTRY_NOT_FOUND_MESSAGE.to_string())))?;

  if found.archived() {
    let keep_existing_link = allow_archived_if_same_as == Some(*address_entry_id);
    if !keep_existing_link {
      return Err(postcard_command_error(AppError::Validation(
        ADDRESS_ENTRY_ARCHIVED_MESSAGE.to_string(),
      )));
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
        Uuid::parse_str(&id).map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
      validate_address_entry_for_receipt(pool, &uuid, allow_archived_if_same_as).await?;
      Some(uuid)
    }
    None => None,
  };

  let received_at = chrono::NaiveDate::parse_from_str(&dto.received_at, "%Y-%m-%d")
    .map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
  let category = PostcardReceiptCategory::parse(&dto.category)
    .map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
  let memo = match dto.memo {
    Some(text) if !text.is_empty() => Some(
      crate::domain::address::memo::Memo::new(text)
        .map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?,
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
  .map_err(|e| postcard_command_error(map_postcard_receipt_error(e)))?;

  let id = receipt.id().as_uuid().to_string();
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());
  repo
    .create(&receipt, None)
    .await
    .map_err(|e| map_postcard_receipt_write_error(e, "create_postcard_receipt", "RECEIPT_CREATE_FAILED"))?;
  Ok(id)
}

#[tauri::command]
async fn update_postcard_receipt(
  pool: State<'_, SqlitePool>,
  id: String,
  dto: PostcardReceiptDtoInput,
  expected_updated_at: String,
) -> Result<(), String> {
  update_postcard_receipt_impl(pool.inner(), id, dto, expected_updated_at).await
}

async fn update_postcard_receipt_impl(
  pool: &SqlitePool,
  id: String,
  dto: PostcardReceiptDtoInput,
  expected_updated_at: String,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
  let receipt_id = PostcardReceiptId::from_uuid(uuid);
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());

  DateTime::parse_from_rfc3339(&expected_updated_at)
    .map_err(|e| postcard_command_error(AppError::Validation(format!("invalid expected_updated_at: {e}"))))?;

  let existing = repo
    .find_by_id(&receipt_id)
    .await
    .map_err(|e| {
      log::error!("update_postcard_receipt find_by_id failed: {:?}", e);
      AppError::Repository("RECEIPT_UPDATE_FAILED".to_string())
    })?
    .ok_or_else(|| AppError::Validation(RECEIPT_NOT_FOUND_MESSAGE.to_string()))?;

  if existing.receipt.is_deleted() {
    return Err(postcard_command_error(AppError::Validation(
      RECEIPT_NOT_FOUND_MESSAGE.to_string(),
    )));
  }

  let existing_address_id = existing.receipt.address_entry_id();
  let previous_received_at = existing.receipt.received_at();

  let (address_entry_id, sender_display_name, received_at, category, memo) =
    build_postcard_receipt_values_from_input(
      pool,
      dto,
      existing_address_id,
    )
    .await?;

  let receipt = PostcardReceipt::from_persisted_for_update(
    receipt_id,
    address_entry_id,
    sender_display_name,
    received_at,
    category,
    memo,
    existing.receipt.deleted_at(),
    existing.receipt.created_at(),
    Utc::now(),
    previous_received_at,
    PostcardReceipt::local_today(),
  )
  .map_err(|e| postcard_command_error(map_postcard_receipt_error(e)))?;

  repo
    .update(&receipt, existing_address_id, &expected_updated_at)
    .await
    .map_err(|e| map_postcard_receipt_write_error(e, "update_postcard_receipt", "RECEIPT_UPDATE_FAILED"))?;
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
    Uuid::parse_str(&id).map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
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
    return Err(postcard_command_error(AppError::Validation(
      RECEIPT_NOT_FOUND_MESSAGE.to_string(),
    )));
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
  limit: Option<i64>,
  offset: Option<i64>,
  sort_order: Option<String>,
) -> Result<PostcardReceiptSearchResult, String> {
  search_postcard_receipts_impl(
    pool.inner(),
    keyword,
    year,
    category,
    address_entry_id,
    limit,
    offset,
    sort_order,
  )
  .await
}

async fn search_postcard_receipts_impl(
  pool: &SqlitePool,
  keyword: Option<String>,
  year: Option<i32>,
  category: Option<String>,
  address_entry_id: Option<String>,
  limit: Option<i64>,
  offset: Option<i64>,
  sort_order: Option<String>,
) -> Result<PostcardReceiptSearchResult, String> {
  let (l, o) = (
    limit.unwrap_or(DEFAULT_SEARCH_LIMIT),
    offset.unwrap_or(DEFAULT_SEARCH_OFFSET),
  );
  if l < 1 || l > MAX_PAGE_LIMIT {
    return Err(postcard_command_error(AppError::Validation(format!(
      "limit must be between 1 and {}",
      MAX_PAGE_LIMIT
    ))));
  }
  if o < 0 {
    return Err(postcard_command_error(AppError::Validation(
      "offset must be >= 0".to_string(),
    )));
  }

  let parsed_category = match category {
    Some(value) if !value.is_empty() => Some(
      PostcardReceiptCategory::parse(&value)
        .map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?,
    ),
    _ => None,
  };

  let parsed_address_entry_id = match address_entry_id {
    Some(id) if !id.is_empty() => Some(
      Uuid::parse_str(&id).map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?,
    ),
    _ => None,
  };

  let sort_order = match sort_order.as_deref() {
    Some("asc") => ReceiptSortOrder::Asc,
    _ => ReceiptSortOrder::Desc,
  };

  // 削除済み一覧・復元は v1 非スコープのため、公開 API は常に active のみ返す
  let query = PostcardReceiptSearchQuery {
    keyword: keyword.filter(|k| !k.trim().is_empty()),
    year,
    category: parsed_category,
    address_entry_id: parsed_address_entry_id,
    include_deleted: false,
    pagination: ReceiptPagination { limit: l, offset: o },
    sort_order,
  };

  let repo = SqlxPostcardReceiptRepository::new(pool.clone());
  let (items, total) = repo.search(query).await.map_err(|e| {
    log::error!("search_postcard_receipts failed: {:?}", e);
    String::from(AppError::Repository("RECEIPT_SEARCH_FAILED".to_string()))
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
async fn list_postcard_receipt_years(pool: State<'_, SqlitePool>) -> Result<Vec<i32>, String> {
  list_postcard_receipt_years_impl(pool.inner()).await
}

async fn list_postcard_receipt_years_impl(pool: &SqlitePool) -> Result<Vec<i32>, String> {
  let repo = SqlxPostcardReceiptRepository::new(pool.clone());
  repo.list_received_years().await.map_err(|e| {
    log::error!("list_postcard_receipt_years failed: {:?}", e);
    String::from(AppError::Repository("RECEIPT_LIST_YEARS_FAILED".to_string()))
  })
}

#[tauri::command]
async fn delete_postcard_receipt(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
  delete_postcard_receipt_impl(pool.inner(), id).await
}

async fn delete_postcard_receipt_impl(pool: &SqlitePool, id: String) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| postcard_command_error(AppError::Validation(e.to_string())))?;
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

// ---------------------------------------------------------------------------
// Print (TOP-28)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CoRecipientPrintDto {
  pub last: String,
  pub first: String,
  pub omit_last: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AddressPrintSnapshotDto {
  pub address_entry_id: String,
  pub postal_code: String,
  pub address_line1: String,
  pub address_line2: String,
  pub address_line3: String,
  pub primary_last: String,
  pub primary_first: String,
  pub co_recipients: Vec<CoRecipientPrintDto>,
  pub honorific_print: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SenderPrintSnapshotDto {
  pub sender_entry_id: String,
  pub postal_code: String,
  pub address_line1: String,
  pub address_line2: String,
  pub address_line3: String,
  pub primary_last: String,
  pub primary_first: String,
  pub co_recipients: Vec<CoRecipientPrintDto>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PrintJobItemDto {
  pub address: AddressPrintSnapshotDto,
  pub sender: SenderPrintSnapshotDto,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ExcludedAlertDto {
  pub address_entry_id: String,
  /// `"no_sender_link"` | `"sender_archived"`
  pub reason: String,
  /// 確認画面表示用（無い場合はフロントが ID にフォールバック）
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub display_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ResolvePrintJobItemsResult {
  pub items: Vec<PrintJobItemDto>,
  pub excluded: Vec<ExcludedAlertDto>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PrintLayoutPreferenceDto {
  pub layer_id: String,
  pub offset_x_pt: f64,
  pub offset_y_pt: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CreatePostcardSendItemDto {
  pub address_entry_id: String,
  pub sender_entry_id: String,
  pub address_snapshot: AddressPrintSnapshotDto,
  pub sender_snapshot: SenderPrintSnapshotDto,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CreatePostcardSendsBatchInput {
  pub print_job_id: String,
  /// `"nenga"` | `"mochu"`
  pub postcard_type: String,
  pub items: Vec<CreatePostcardSendItemDto>,
}

/// `resolve_print_job_items` が AddressEntry 側で失敗したときのエラー契約。
///
/// `Err(String)` に次の JSON を載せる（フロントが parse して draft から除外する）:
/// ```json
/// {"code":"ADDRESS_ENTRIES_INVALID","entries":[{"address_entry_id":"...","reason":"archived"|"not_found"}]}
/// ```
/// - `reason`: `"archived"`（存在するが archived） / `"not_found"`（不正 UUID または DB に無い）
/// - 宛名側に 1 件でも該当があればコマンド全体失敗（部分 items は返さない）
fn address_entries_invalid_error(
  entries: &[(String, &'static str)],
) -> String {
  let entry_objs: Vec<serde_json::Value> = entries
    .iter()
    .map(|(id, reason)| {
      serde_json::json!({
        "address_entry_id": id,
        "reason": reason,
      })
    })
    .collect();
  // Validation 経路: AppError::Validation の中身がそのまま String になる
  String::from(AppError::Validation(
    serde_json::json!({
      "code": "ADDRESS_ENTRIES_INVALID",
      "entries": entry_objs,
    })
    .to_string(),
  ))
}

fn map_print_snapshot_error(err: PrintSnapshotError) -> String {
  String::from(AppError::Validation(err.to_string()))
}

fn address_print_snapshot_dto_from(snap: AddressPrintSnapshot) -> AddressPrintSnapshotDto {
  AddressPrintSnapshotDto {
    address_entry_id: snap.address_entry_id,
    postal_code: snap.postal_code,
    address_line1: snap.address_line1,
    address_line2: snap.address_line2,
    address_line3: snap.address_line3,
    primary_last: snap.primary_last,
    primary_first: snap.primary_first,
    co_recipients: snap
      .co_recipients
      .into_iter()
      .map(|c| CoRecipientPrintDto {
        last: c.last,
        first: c.first,
        omit_last: c.omit_last,
      })
      .collect(),
    honorific_print: snap.honorific_print,
  }
}

fn sender_print_snapshot_dto_from(snap: SenderPrintSnapshot) -> SenderPrintSnapshotDto {
  SenderPrintSnapshotDto {
    sender_entry_id: snap.sender_entry_id,
    postal_code: snap.postal_code,
    address_line1: snap.address_line1,
    address_line2: snap.address_line2,
    address_line3: snap.address_line3,
    primary_last: snap.primary_last,
    primary_first: snap.primary_first,
    co_recipients: snap
      .co_recipients
      .into_iter()
      .map(|c| CoRecipientPrintDto {
        last: c.last,
        first: c.first,
        omit_last: c.omit_last,
      })
      .collect(),
  }
}

fn truncate_print_ids(ids: Vec<String>) -> Vec<String> {
  let mut seen = std::collections::HashSet::new();
  let mut out = Vec::with_capacity(ids.len().min(MAX_PRINT_ADDRESS_ENTRY_IDS));
  for id in ids {
    if seen.insert(id.clone()) {
      out.push(id);
      if out.len() >= MAX_PRINT_ADDRESS_ENTRY_IDS {
        break;
      }
    }
  }
  out
}

fn is_allowed_print_layer_id(layer_id: &str) -> bool {
  PRINT_LAYER_ID_ALLOWLIST.contains(&layer_id)
}

fn address_entry_display_name(entry: &AddressEntry) -> String {
  format!(
    "{} {}",
    entry.primary_name().last(),
    entry.primary_name().first()
  )
  .trim()
  .to_string()
}

#[tauri::command]
async fn filter_active_address_entry_ids(
  pool: State<'_, SqlitePool>,
  address_entry_ids: Vec<String>,
) -> Result<Vec<String>, String> {
  filter_active_address_entry_ids_impl(pool.inner(), address_entry_ids).await
}

/// 入力順を保った非 archived 存在 ID の subset。
/// 不正 UUID / not found / archived は落とす（エラーにしない）。IPC/DB 失敗のみエラー。
async fn filter_active_address_entry_ids_impl(
  pool: &SqlitePool,
  address_entry_ids: Vec<String>,
) -> Result<Vec<String>, String> {
  let address_entry_ids = truncate_print_ids(address_entry_ids);
  if address_entry_ids.is_empty() {
    return Ok(vec![]);
  }

  let mut valid_ordered: Vec<String> = Vec::new();
  let mut uuid_strings: Vec<String> = Vec::new();
  for id_str in &address_entry_ids {
    if Uuid::parse_str(id_str).is_err() {
      continue;
    }
    valid_ordered.push(id_str.clone());
    uuid_strings.push(id_str.clone());
  }
  if uuid_strings.is_empty() {
    return Ok(vec![]);
  }

  let placeholders = uuid_strings
    .iter()
    .enumerate()
    .map(|(i, _)| format!("?{}", i + 1))
    .collect::<Vec<_>>()
    .join(", ");
  let sql = format!(
    "SELECT id FROM address_entries WHERE archived_at IS NULL AND id IN ({placeholders})"
  );
  let mut query = sqlx::query(&sql);
  for id in &uuid_strings {
    query = query.bind(id);
  }
  let rows = query.fetch_all(pool).await.map_err(|e| {
    log::error!("filter_active_address_entry_ids failed: {:?}", e);
    String::from(AppError::Repository("PRINT_FILTER_ACTIVE_FAILED".to_string()))
  })?;

  let active: std::collections::HashSet<String> = rows
    .into_iter()
    .map(|row| row.get::<String, _>("id"))
    .collect();

  Ok(
    valid_ordered
      .into_iter()
      .filter(|id| active.contains(id))
      .collect(),
  )
}

#[tauri::command]
async fn resolve_print_job_items(
  pool: State<'_, SqlitePool>,
  address_entry_ids: Vec<String>,
) -> Result<ResolvePrintJobItemsResult, String> {
  resolve_print_job_items_impl(pool.inner(), address_entry_ids).await
}

async fn resolve_print_job_items_impl(
  pool: &SqlitePool,
  address_entry_ids: Vec<String>,
) -> Result<ResolvePrintJobItemsResult, String> {
  use crate::infrastructure::address::sqlx_address_entry_repository::build_entries_with_co_recipients;

  let address_entry_ids = truncate_print_ids(address_entry_ids);
  let sender_repo = SqlxSenderEntryRepository::new(pool.clone());

  let mut offending: Vec<(String, &'static str)> = Vec::new();
  let mut ordered_valid: Vec<String> = Vec::new();
  for id_str in &address_entry_ids {
    if Uuid::parse_str(id_str).is_err() {
      offending.push((id_str.clone(), "not_found"));
      continue;
    }
    ordered_valid.push(id_str.clone());
  }

  let mut entry_by_id: std::collections::HashMap<String, AddressEntry> =
    std::collections::HashMap::new();
  if !ordered_valid.is_empty() {
    const IN_CHUNK_SIZE: usize = 100;
    for chunk in ordered_valid.chunks(IN_CHUNK_SIZE) {
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
      let rows = q.fetch_all(pool).await.map_err(|e| {
        log::error!("resolve_print_job_items address lookup failed: {:?}", e);
        String::from(AppError::Repository("PRINT_RESOLVE_FAILED".to_string()))
      })?;
      let entries = build_entries_with_co_recipients(rows, pool)
        .await
        .map_err(|e| {
          log::error!("resolve_print_job_items address assemble failed: {:?}", e);
          String::from(AppError::Repository("PRINT_RESOLVE_FAILED".to_string()))
        })?;
      for entry in entries {
        entry_by_id.insert(entry.id().as_uuid().to_string(), entry);
      }
    }
  }

  let mut resolved_entries: Vec<(String, AddressEntry)> = Vec::new();
  for id_str in &ordered_valid {
    match entry_by_id.remove(id_str) {
      None => offending.push((id_str.clone(), "not_found")),
      Some(e) if e.archived() => offending.push((id_str.clone(), "archived")),
      Some(e) => resolved_entries.push((id_str.clone(), e)),
    }
  }

  if !offending.is_empty() {
    return Err(address_entries_invalid_error(&offending));
  }

  // 差出人リンクをバッチ取得
  let mut link_by_address: std::collections::HashMap<String, String> =
    std::collections::HashMap::new();
  if !resolved_entries.is_empty() {
    let addr_ids: Vec<String> = resolved_entries.iter().map(|(id, _)| id.clone()).collect();
    const IN_CHUNK_SIZE: usize = 100;
    for chunk in addr_ids.chunks(IN_CHUNK_SIZE) {
      let placeholders = chunk
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
      let sql = format!(
        r#"
          SELECT address_entry_id, sender_entry_id, updated_at
          FROM sender_address_links
          WHERE address_entry_id IN ({placeholders})
          ORDER BY updated_at DESC
        "#
      );
      let mut q = sqlx::query(&sql);
      for id in chunk {
        q = q.bind(id);
      }
      let rows = q.fetch_all(pool).await.map_err(|e| {
        log::error!("resolve_print_job_items sender link lookup failed: {:?}", e);
        String::from(AppError::Repository("PRINT_RESOLVE_FAILED".to_string()))
      })?;
      for row in rows {
        let address_id: String = row.get("address_entry_id");
        let sender_id: String = row.get("sender_entry_id");
        // ORDER BY updated_at DESC なので初出が最新
        link_by_address.entry(address_id).or_insert(sender_id);
      }
    }
  }

  let mut items = Vec::new();
  let mut excluded = Vec::new();

  for (id_str, address_entry) in resolved_entries {
    let display_name = Some(address_entry_display_name(&address_entry));
    let Some(sender_id_str) = link_by_address.get(&id_str) else {
      excluded.push(ExcludedAlertDto {
        address_entry_id: id_str,
        reason: "no_sender_link".to_string(),
        display_name,
      });
      continue;
    };

    let Ok(sender_uuid) = Uuid::parse_str(sender_id_str) else {
      excluded.push(ExcludedAlertDto {
        address_entry_id: id_str,
        reason: "no_sender_link".to_string(),
        display_name,
      });
      continue;
    };

    let sender_entry = sender_repo
      .find_by_id(&SenderEntryId::from_uuid(sender_uuid))
      .await
      .map_err(|e| {
        log::error!("resolve_print_job_items sender lookup failed: {:?}", e);
        String::from(AppError::Repository("PRINT_RESOLVE_FAILED".to_string()))
      })?;

    let Some(sender_entry) = sender_entry else {
      excluded.push(ExcludedAlertDto {
        address_entry_id: id_str,
        reason: "no_sender_link".to_string(),
        display_name,
      });
      continue;
    };

    if sender_entry.archived() {
      excluded.push(ExcludedAlertDto {
        address_entry_id: id_str,
        reason: "sender_archived".to_string(),
        display_name,
      });
      continue;
    }

    let address_snap = AddressPrintSnapshot::from_address_entry(&address_entry)
      .map_err(map_print_snapshot_error)?;
    let sender_snap =
      SenderPrintSnapshot::from_sender_entry(&sender_entry).map_err(map_print_snapshot_error)?;

    items.push(PrintJobItemDto {
      address: address_print_snapshot_dto_from(address_snap),
      sender: sender_print_snapshot_dto_from(sender_snap),
    });
  }

  Ok(ResolvePrintJobItemsResult { items, excluded })
}

#[tauri::command]
async fn build_address_print_snapshot(
  pool: State<'_, SqlitePool>,
  address_entry_id: String,
) -> Result<AddressPrintSnapshotDto, String> {
  build_address_print_snapshot_impl(pool.inner(), address_entry_id).await
}

async fn build_address_print_snapshot_impl(
  pool: &SqlitePool,
  address_entry_id: String,
) -> Result<AddressPrintSnapshotDto, String> {
  let uuid = Uuid::parse_str(&address_entry_id)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
  let repo = SqlxAddressEntryRepository::new(pool.clone());
  let entry = repo
    .find_by_id(&AddressEntryId::from_uuid(uuid))
    .await
    .map_err(|e| {
      log::error!("build_address_print_snapshot failed: {:?}", e);
      String::from(AppError::Repository("PRINT_SNAPSHOT_FAILED".to_string()))
    })?
    .ok_or_else(|| String::from(AppError::Validation(ADDRESS_ENTRY_NOT_FOUND_MESSAGE.to_string())))?;

  if entry.archived() {
    return Err(String::from(AppError::Validation(
      ADDRESS_ENTRY_ARCHIVED_MESSAGE.to_string(),
    )));
  }

  let snap =
    AddressPrintSnapshot::from_address_entry(&entry).map_err(map_print_snapshot_error)?;
  Ok(address_print_snapshot_dto_from(snap))
}

#[tauri::command]
async fn build_sender_print_snapshot(
  pool: State<'_, SqlitePool>,
  sender_entry_id: String,
) -> Result<SenderPrintSnapshotDto, String> {
  build_sender_print_snapshot_impl(pool.inner(), sender_entry_id).await
}

async fn build_sender_print_snapshot_impl(
  pool: &SqlitePool,
  sender_entry_id: String,
) -> Result<SenderPrintSnapshotDto, String> {
  let uuid = Uuid::parse_str(&sender_entry_id)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
  let repo = SqlxSenderEntryRepository::new(pool.clone());
  let entry = repo
    .find_by_id(&SenderEntryId::from_uuid(uuid))
    .await
    .map_err(|e| {
      log::error!("build_sender_print_snapshot failed: {:?}", e);
      String::from(AppError::Repository("PRINT_SNAPSHOT_FAILED".to_string()))
    })?
    .ok_or_else(|| String::from(AppError::Validation(SENDER_ENTRY_NOT_FOUND_MESSAGE.to_string())))?;

  if entry.archived() {
    return Err(String::from(AppError::Validation(
      SENDER_ENTRY_ARCHIVED_MESSAGE.to_string(),
    )));
  }

  let snap = SenderPrintSnapshot::from_sender_entry(&entry).map_err(map_print_snapshot_error)?;
  Ok(sender_print_snapshot_dto_from(snap))
}

#[tauri::command]
async fn list_print_layout_preferences(
  pool: State<'_, SqlitePool>,
  postcard_type: String,
) -> Result<Vec<PrintLayoutPreferenceDto>, String> {
  list_print_layout_preferences_impl(pool.inner(), postcard_type).await
}

async fn list_print_layout_preferences_impl(
  pool: &SqlitePool,
  postcard_type: String,
) -> Result<Vec<PrintLayoutPreferenceDto>, String> {
  let postcard_type = PostcardType::from_str(&postcard_type)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
  let repo = SqlxPrintLayoutPreferenceRepository::new(pool.clone());
  let prefs = repo.list_by_postcard_type(postcard_type).await.map_err(|e| {
    log::error!("list_print_layout_preferences failed: {:?}", e);
    String::from(AppError::Repository("PRINT_LAYOUT_LIST_FAILED".to_string()))
  })?;
  Ok(
    prefs
      .into_iter()
      .map(|p| PrintLayoutPreferenceDto {
        layer_id: p.layer_id().to_string(),
        offset_x_pt: p.offset_x_pt(),
        offset_y_pt: p.offset_y_pt(),
      })
      .collect(),
  )
}

#[tauri::command]
async fn save_print_layout_preferences(
  pool: State<'_, SqlitePool>,
  postcard_type: String,
  offsets: Vec<PrintLayoutPreferenceDto>,
) -> Result<(), String> {
  save_print_layout_preferences_impl(pool.inner(), postcard_type, offsets).await
}

async fn save_print_layout_preferences_impl(
  pool: &SqlitePool,
  postcard_type: String,
  offsets: Vec<PrintLayoutPreferenceDto>,
) -> Result<(), String> {
  let postcard_type = PostcardType::from_str(&postcard_type)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
  if offsets.len() > PRINT_LAYER_ID_ALLOWLIST.len() {
    return Err(String::from(AppError::Validation(format!(
      "layout offsets exceed max {}",
      PRINT_LAYER_ID_ALLOWLIST.len()
    ))));
  }

  let mut prefs = Vec::with_capacity(offsets.len());
  let mut seen_layers = std::collections::HashSet::new();
  for o in offsets {
    if !is_allowed_print_layer_id(&o.layer_id) {
      return Err(String::from(AppError::Validation(format!(
        "unknown layer_id: {}",
        o.layer_id
      ))));
    }
    if !o.offset_x_pt.is_finite() || !o.offset_y_pt.is_finite() {
      return Err(String::from(AppError::Validation(
        "offset must be a finite number".to_string(),
      )));
    }
    if !seen_layers.insert(o.layer_id.clone()) {
      return Err(String::from(AppError::Validation(format!(
        "duplicate layer_id: {}",
        o.layer_id
      ))));
    }
    prefs.push(PrintLayoutPreference::create_new(
      postcard_type,
      o.layer_id,
      o.offset_x_pt,
      o.offset_y_pt,
    ));
  }

  let repo = SqlxPrintLayoutPreferenceRepository::new(pool.clone());
  repo.save_all(postcard_type, &prefs).await.map_err(|e| {
    log::error!("save_print_layout_preferences failed: {:?}", e);
    String::from(AppError::Repository("PRINT_LAYOUT_SAVE_FAILED".to_string()))
  })?;
  Ok(())
}

#[tauri::command]
async fn create_postcard_sends_batch(
  pool: State<'_, SqlitePool>,
  input: CreatePostcardSendsBatchInput,
) -> Result<(), String> {
  create_postcard_sends_batch_impl(pool.inner(), input).await
}

async fn create_postcard_sends_batch_impl(
  pool: &SqlitePool,
  input: CreatePostcardSendsBatchInput,
) -> Result<(), String> {
  let print_job_id = Uuid::parse_str(&input.print_job_id)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
  let postcard_type = PostcardType::from_str(&input.postcard_type)
    .map_err(|e| String::from(AppError::Validation(e.to_string())))?;

  if input.items.len() > MAX_PRINT_ADDRESS_ENTRY_IDS {
    return Err(String::from(AppError::Validation(format!(
      "print batch exceeds max {} items",
      MAX_PRINT_ADDRESS_ENTRY_IDS
    ))));
  }

  // address_entry_id 重複を除去（同一バッチ内 UNIQUE 衝突の誤冪等を防ぐ）
  let mut seen_address = std::collections::HashSet::new();
  let mut unique_items = Vec::with_capacity(input.items.len());
  for item in input.items {
    if seen_address.insert(item.address_entry_id.clone()) {
      unique_items.push(item);
    }
  }

  let mut sends = Vec::with_capacity(unique_items.len());
  for item in unique_items {
    let address_entry_id = Uuid::parse_str(&item.address_entry_id)
      .map_err(|e| String::from(AppError::Validation(e.to_string())))?;
    let sender_entry_id = Uuid::parse_str(&item.sender_entry_id)
      .map_err(|e| String::from(AppError::Validation(e.to_string())))?;

    let address_snapshot = serde_json::to_string(&item.address_snapshot).map_err(|e| {
      log::error!("create_postcard_sends_batch address snapshot serialize failed: {:?}", e);
      String::from(AppError::Repository("PRINT_SEND_CREATE_FAILED".to_string()))
    })?;
    let sender_snapshot = serde_json::to_string(&item.sender_snapshot).map_err(|e| {
      log::error!("create_postcard_sends_batch sender snapshot serialize failed: {:?}", e);
      String::from(AppError::Repository("PRINT_SEND_CREATE_FAILED".to_string()))
    })?;

    // sent_on は create_new 内で Local::now().date_naive()（フロント非送信）
    sends.push(PostcardSend::create_new(
      print_job_id,
      address_entry_id,
      sender_entry_id,
      sender_snapshot,
      address_snapshot,
      postcard_type,
    ));
  }

  let requested_address_ids: std::collections::HashSet<Uuid> =
    sends.iter().map(|s| s.address_entry_id()).collect();

  let repo = SqlxPostcardSendRepository::new(pool.clone());
  match repo.create_batch(&sends).await {
    Ok(()) => Ok(()),
    Err(PostcardSendRepositoryError::Conflict) => {
      // 正しい再試行のみ冪等成功: 要求 ID がすべて既存であること
      let existing = repo
        .list_address_entry_ids_for_print_job(print_job_id)
        .await
        .map_err(|e| {
          log::error!(
            "create_postcard_sends_batch conflict verify failed: {:?}",
            e
          );
          String::from(AppError::Repository("PRINT_SEND_CREATE_FAILED".to_string()))
        })?;
      let existing_set: std::collections::HashSet<Uuid> = existing.into_iter().collect();
      if !requested_address_ids.is_empty()
        && requested_address_ids
          .iter()
          .all(|id| existing_set.contains(id))
      {
        Ok(())
      } else {
        Err(String::from(AppError::Repository(
          "PRINT_SEND_CREATE_FAILED".to_string(),
        )))
      }
    }
    Err(e) => {
      log::error!("create_postcard_sends_batch failed: {:?}", e);
      Err(String::from(AppError::Repository(
        "PRINT_SEND_CREATE_FAILED".to_string(),
      )))
    }
  }
}
