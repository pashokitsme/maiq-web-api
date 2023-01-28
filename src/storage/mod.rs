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

impl Into<Snapshot> for SnapshotModel {
  fn into(self) -> Snapshot {
    Snapshot { uid: self.uid, date: self.date.to_chrono(), parsed_date: self.parsed_date.to_chrono(), groups: self.groups }
  }
}

impl Into<Option<Snapshot>> for SnapshotModel {
  fn into(self) -> Option<Snapshot> {
    Some(self.into())
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
