pub mod commands;
pub mod queries;

use mongodb::{options::ClientOptions, Client, Collection};

use crate::env;

pub use commands::*;
pub use queries::*;
pub type MongoPool = Client;
pub type MongoError = mongodb::error::Error;

use maiq_parser::{Group, Snapshot};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};

// todo: validate collections on init
pub async fn init() -> Result<MongoPool, MongoError> {
  let url = dotenvy::var(env::DB_URL).unwrap();
  info!("Connecting to {}", url);

  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq-web".into());
  opts.default_database = Some(env::var(env::DEFAULT_DB).unwrap());

  let client = MongoPool::with_options(opts)?;

  Ok(client)
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

impl From<Snapshot> for SnapshotModel {
  fn from(s: Snapshot) -> Self {
    Self { uid: s.uid, date: s.date.into(), parsed_date: s.parsed_date.into(), groups: s.groups }
  }
}

fn get_snapshots_as_model(mongo: &MongoPool) -> Collection<SnapshotModel> {
  mongo.default_database().unwrap().collection("snapshots")
}
