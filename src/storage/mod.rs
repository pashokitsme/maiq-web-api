pub mod cache;
pub mod mongo;

use mongodb::Collection;

use crate::api::error::ApiError;

use maiq_parser::{Fetch, Group, Snapshot};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};

use self::mongo::MongoPool;

#[rocket::async_trait]
pub trait SnapshotPool {
  async fn save(&mut self, snapshot: &Snapshot) -> Result<(), ApiError>;
  async fn latest(&self, mode: Fetch) -> Result<Option<Snapshot>, ApiError>;
  async fn by_uid<T: AsRef<str> + Send>(&self, uid: T) -> Result<Option<Snapshot>, ApiError>;
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotModel {
  pub uid: String,
  pub date: DateTime,
  pub parsed_date: DateTime,
  pub groups: Vec<Group>,
}

impl From<SnapshotModel> for Snapshot {
  fn from(val: SnapshotModel) -> Self {
    Snapshot { uid: val.uid, date: val.date.to_chrono(), parsed_date: val.parsed_date.to_chrono(), groups: val.groups }
  }
}

impl From<SnapshotModel> for Option<Snapshot> {
  fn from(val: SnapshotModel) -> Self {
    Some(val.into())
  }
}

impl From<&Snapshot> for SnapshotModel {
  fn from(s: &Snapshot) -> Self {
    Self { uid: s.uid.clone(), date: s.date.into(), parsed_date: s.parsed_date.into(), groups: s.groups.clone() }
  }
}

impl MongoPool {
  fn get_snapshot_models(&self) -> Collection<SnapshotModel> {
    self.default_database().unwrap().collection("snapshots")
  }
}
