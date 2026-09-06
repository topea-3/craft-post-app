use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::print::postcard_type::PostcardType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrintLayoutPreferenceId(Uuid);

impl PrintLayoutPreferenceId {
  pub fn new() -> Self {
    Self(Uuid::new_v4())
  }

  pub fn from_uuid(uuid: Uuid) -> Self {
    Self(uuid)
  }

  pub fn as_uuid(&self) -> Uuid {
    self.0
  }
}

#[derive(Debug, Clone)]
pub struct PrintLayoutPreference {
  id: PrintLayoutPreferenceId,
  postcard_type: PostcardType,
  layer_id: String,
  offset_x_pt: f64,
  offset_y_pt: f64,
  updated_at: DateTime<Utc>,
}

impl PrintLayoutPreference {
  pub fn create_new(
    postcard_type: PostcardType,
    layer_id: String,
    offset_x_pt: f64,
    offset_y_pt: f64,
  ) -> Self {
    Self {
      id: PrintLayoutPreferenceId::new(),
      postcard_type,
      layer_id,
      offset_x_pt,
      offset_y_pt,
      updated_at: Utc::now(),
    }
  }

  pub fn from_persisted(
    id: PrintLayoutPreferenceId,
    postcard_type: PostcardType,
    layer_id: String,
    offset_x_pt: f64,
    offset_y_pt: f64,
    updated_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id,
      postcard_type,
      layer_id,
      offset_x_pt,
      offset_y_pt,
      updated_at,
    }
  }

  pub fn id(&self) -> &PrintLayoutPreferenceId {
    &self.id
  }

  pub fn postcard_type(&self) -> PostcardType {
    self.postcard_type
  }

  pub fn layer_id(&self) -> &str {
    &self.layer_id
  }

  pub fn offset_x_pt(&self) -> f64 {
    self.offset_x_pt
  }

  pub fn offset_y_pt(&self) -> f64 {
    self.offset_y_pt
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }
}
