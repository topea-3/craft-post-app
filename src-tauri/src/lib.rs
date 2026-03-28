mod domain;
mod infrastructure;
#[cfg(test)]
mod command_tests;

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
  // sender の実在 + archived = 0 を検証
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

  // address_entries の実在 + archived = 0 を検証（0件は解除扱いなのでOK）
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
  let sender_uuid =
    Uuid::parse_str(&sender_id).map_err(|e| AppError::Validation(e.to_string()).to_string())?;
  let sender_entry_id = SenderEntryId::from_uuid(sender_uuid);

  let repo = SqlxSenderEntryRepository::new(pool.inner().clone());
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
  // address の実在 + archived = 0 を検証
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
  // sender の実在 + archived = 0 を検証（Some の場合）
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
      "SELECT COUNT(*) AS cnt FROM address_entries WHERE archived = 0 AND id IN ({})",
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
