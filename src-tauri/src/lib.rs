mod domain;
mod infrastructure;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
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
use crate::infrastructure::address::sqlx_address_entry_repository::SqlxAddressEntryRepository;

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
      archive_address_entry
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
    err.to_string()
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

impl TryFrom<AddressEntryDtoInput> for AddressEntry {
  type Error = AppError;

  fn try_from(value: AddressEntryDtoInput) -> Result<Self, Self::Error> {
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
    .map_err(|e| AppError::Repository(e.to_string()))?;
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
    .map_err(|e| AppError::Repository(e.to_string()))?
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
    .map_err(|e| AppError::Repository(e.to_string()))?;
  Ok(())
}

#[tauri::command]
async fn list_address_entries(
  pool: State<'_, SqlitePool>,
  limit: i64,
  offset: i64,
) -> Result<Vec<AddressEntryDto>, String> {
  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let entries = repo
    .list_active(Pagination { limit, offset })
    .await
    .map_err(|e| AppError::Repository(e.to_string()))?;

  Ok(entries.into_iter().map(AddressEntryDto::from).collect())
}

#[tauri::command]
async fn search_address_entries(
  pool: State<'_, SqlitePool>,
  keyword: Option<String>,
  sort_key: String,
  sort_order: String,
  include_archived: bool,
  limit: Option<i64>,
  offset: Option<i64>,
) -> Result<Vec<AddressEntryDto>, String> {
  let sort_key = match sort_key.as_str() {
    "updated_at" => SortKey::UpdatedAt,
    _ => SortKey::NameKana,
  };
  let sort_order = match sort_order.as_str() {
    "desc" => SortOrder::Desc,
    _ => SortOrder::Asc,
  };
  let pagination = match (limit, offset) {
    (Some(l), Some(o)) => Some(Pagination { limit: l, offset: o }),
    _ => None,
  };

  let query = AddressSearchQuery {
    keyword,
    sort_key,
    sort_order,
    include_archived,
    pagination,
  };

  let repo = SqlxAddressEntryRepository::new(pool.inner().clone());
  let entries = repo
    .search(query)
    .await
    .map_err(|e| AppError::Repository(e.to_string()))?;

  Ok(entries.into_iter().map(AddressEntryDto::from).collect())
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
    .map_err(|e| AppError::Repository(e.to_string()))?;
  Ok(())
}
