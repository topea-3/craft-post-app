mod domain;
mod infrastructure;

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tauri::{Manager, State};
use uuid::Uuid;

use crate::domain::address::address::Address;
use crate::domain::address::address_entry::{AddressEntry, AddressEntryId};
use crate::domain::address::address_entry_repository::{
  AddressEntryRepository, AddressSearchQuery, Pagination, SortKey, SortOrder,
};
use crate::domain::address::honorific::Honorific;
use crate::domain::address::memo::Memo;
use crate::domain::address::person_name::PersonName;
use crate::domain::address::postal_code::PostalCode;
use crate::domain::sender::phone_number::PhoneNumber;
use crate::domain::sender::sender_entry::{SenderEntry, SenderEntryId};
use crate::domain::sender::sender_entry_repository::{
  Pagination as SenderPagination, SenderEntryRepository,
};
use crate::domain::sender::sender_label::SenderLabel;
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;
use crate::infrastructure::sender::sqlx_sender_entry_repository::SqlxSenderEntryRepository;

const MAX_PAGE_LIMIT: i64 = 200;
/// 連名の上限（UI と同一。API 直叩き対策でサーバー側でも検証する）
const MAX_CO_RECIPIENTS: usize = 3;
const MAX_SENDER_CO_RECIPIENTS: usize = 4;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

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
      search_unlinked_address_entries,
      set_sender_for_address_entry,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
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

  // 既存エントリを取得し、created_at / archived を維持する。
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
    existing.archived(),
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
  let entry =
    SenderEntry::try_from(dto).map_err::<String, _>(|e| e.into()).map_err(|e| e.to_string())?;
  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  let duplicated = repo
    .exists_active_label(entry.label().value(), None)
    .await
    .map_err(|e| {
      log::error!("create_sender_entry exists_active_label failed: {:?}", e);
      AppError::Repository("SENDER_CREATE_FAILED".to_string())
    })?;
  if duplicated {
    return Err(String::from(AppError::Validation(
      "このラベルは既に使用されています。別のラベルを指定してください。".to_string(),
    )));
  }
  repo
    .create(&entry)
    .await
    .map_err(|e| {
      log::error!("create_sender_entry failed: {:?}", e);
      AppError::Repository("SENDER_CREATE_FAILED".to_string())
    })?;
  Ok(())
}

#[tauri::command]
async fn update_sender_entry(
  pool: State<'_, SqlitePool>,
  id: String,
  dto: SenderEntryDtoInput,
) -> Result<(), String> {
  let uuid =
    Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let id = SenderEntryId::from_uuid(uuid);
  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());

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
      "このラベルは既に使用されています。別のラベルを指定してください。".to_string(),
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
    existing.archived(),
    existing.created_at(),
    now,
  )
  .map_err(|e| AppError::Validation(e.to_string()).to_string())?;

  repo
    .update(&entry)
    .await
    .map_err(|e| {
      log::error!("update_sender_entry failed: {:?}", e);
      AppError::Repository("SENDER_UPDATE_FAILED".to_string())
    })?;
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
  let sender_uuid =
    Uuid::parse_str(&sender_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = SenderEntryId::from_uuid(sender_uuid);

  let mut parsed_address_ids = Vec::with_capacity(address_entry_ids.len());
  for id in address_entry_ids {
    let parsed =
      Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
    parsed_address_ids.push(parsed);
  }

  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
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
  let sender_uuid =
    Uuid::parse_str(&sender_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = SenderEntryId::from_uuid(sender_uuid);

  let sender_repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  let address_ids = sender_repo
    .list_linked_address_entry_ids(&sender_entry_id)
    .await
    .map_err(|e| {
      log::error!("list_sender_linked_addresses list_linked_address_entry_ids failed: {:?}", e);
      AppError::Repository("SENDER_LINK_LIST_FAILED".to_string())
    })?;

  let address_repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let mut result = Vec::with_capacity(address_ids.len());
  for addr_id in address_ids {
    let id = AddressEntryId::from_uuid(addr_id);
    let Some(entry) = address_repo
      .find_by_id(&id)
      .await
      .map_err(|e| {
        log::error!("list_sender_linked_addresses find_by_id failed: {:?}", e);
        AppError::Repository("SENDER_LINK_LIST_FAILED".to_string())
      })?
    else {
      continue;
    };
    result.push(AddressEntryDto::from(entry));
  }

  Ok(result)
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
async fn search_unlinked_address_entries(
  pool: State<'_, SqlitePool>,
  keyword: Option<String>,
  limit: Option<i64>,
  offset: Option<i64>,
) -> Result<AddressEntrySearchResult, String> {
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

  let mut where_sql = String::from(
    r#"
      WHERE archived = 0
        AND NOT EXISTS (
          SELECT 1 FROM sender_address_links sal
          WHERE sal.address_entry_id = address_entries.id
        )
    "#,
  );
  if keyword.is_some() {
    where_sql.push_str(
      r#"
        AND (
          primary_last       LIKE ? OR
          primary_first      LIKE ? OR
          primary_kana_last  LIKE ? OR
          primary_kana_first LIKE ? OR
          prefecture || city || street || IFNULL(building, '') LIKE ? OR
          IFNULL(memo, '') LIKE ?
        )
      "#,
    );
  }

  // 総件数
  let count_sql = format!("SELECT COUNT(*) AS cnt FROM address_entries {}", where_sql);
  let mut count_q = sqlx::query(&count_sql);
  if let Some(k) = keyword.as_ref() {
    let kw = format!("%{}%", k);
    for _ in 0..6 {
      count_q = count_q.bind(kw.clone());
    }
  }
  let total: i64 = count_q
    .fetch_one(pool.inner())
    .await
    .map_err(|e| {
      log::error!("search_unlinked_address_entries count failed: {:?}", e);
      AppError::Repository("ADDR_SEARCH_FAILED".to_string())
    })?
    .get("cnt");

  // ID 一覧（updated_at desc 固定）
  let list_sql = format!(
    r#"
      SELECT id
      FROM address_entries
      {}
      ORDER BY updated_at DESC, id ASC
      LIMIT ? OFFSET ?
    "#,
    where_sql
  );
  let mut q = sqlx::query(&list_sql);
  if let Some(k) = keyword.as_ref() {
    let kw = format!("%{}%", k);
    for _ in 0..6 {
      q = q.bind(kw.clone());
    }
  }
  q = q.bind(l).bind(o);

  let rows = q.fetch_all(pool.inner()).await.map_err(|e| {
    log::error!("search_unlinked_address_entries list failed: {:?}", e);
    AppError::Repository("ADDR_SEARCH_FAILED".to_string())
  })?;

  let address_repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let mut items = Vec::with_capacity(rows.len());
  for r in rows {
    let id: String = r.get("id");
    let uuid = Uuid::parse_str(&id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
    let id = AddressEntryId::from_uuid(uuid);
    let Some(entry) = address_repo.find_by_id(&id).await.map_err(|e| {
      log::error!("search_unlinked_address_entries find_by_id failed: {:?}", e);
      AppError::Repository("ADDR_SEARCH_FAILED".to_string())
    })? else {
      continue;
    };
    items.push(AddressEntryDto::from(entry));
  }

  Ok(AddressEntrySearchResult { items, total })
}

#[tauri::command]
async fn set_sender_for_address_entry(
  pool: State<'_, SqlitePool>,
  address_entry_id: String,
  sender_id: Option<String>,
) -> Result<(), String> {
  let address_uuid =
    Uuid::parse_str(&address_entry_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = match sender_id {
    Some(s) => {
      let uuid =
        Uuid::parse_str(&s).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
      Some(SenderEntryId::from_uuid(uuid))
    }
    None => None,
  };

  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
  repo
    .set_sender_for_address(address_uuid, sender_entry_id.as_ref())
    .await
    .map_err(|e| {
      log::error!("set_sender_for_address_entry failed: {:?}", e);
      AppError::Repository("SENDER_LINK_UPDATE_FAILED".to_string())
    })?;

  Ok(())
}
