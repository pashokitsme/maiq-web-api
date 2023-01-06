pub mod commands;
pub mod queries;

use std::ops::Deref;

use mongodb::{options::ClientOptions, Client, Collection};

use crate::env;

pub use commands::*;
pub use queries::*;
pub type MongoClient = Client;
pub type MongoError = mongodb::error::Error;

use maiq_parser::{Group, Snapshot};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MongoPool {
  client: MongoClient,
}

impl Deref for MongoPool {
  type Target = MongoClient;

  fn deref(&self) -> &Self::Target {
    &self.client
  }
}

// todo: validate collections on init
pub async fn init() -> Result<MongoPool, MongoError> {
  let url = dotenvy::var(env::DB_URL).unwrap();
  info!("Connecting to database..");

  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq-web".into());
  opts.default_database = Some(env::var(env::DEFAULT_DB).unwrap());

  let client = MongoClient::with_options(opts)?;

  Ok(MongoPool { client })
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotModel {
  pub uid: String,
  pub is_week_even: bool,
  pub date: DateTime,
  pub parsed_date: DateTime,
  pub groups: Vec<Group>,
}

impl Into<Snapshot> for SnapshotModel {
  fn into(self) -> Snapshot {
    Snapshot {
      uid: self.uid,
      is_week_even: self.is_week_even,
      date: self.date.to_chrono(),
      parsed_date: self.parsed_date.to_chrono(),
      groups: self.groups,
    }
  }
}

impl From<Snapshot> for SnapshotModel {
  fn from(s: Snapshot) -> Self {
    Self { uid: s.uid, is_week_even: s.is_week_even, date: s.date.into(), parsed_date: s.parsed_date.into(), groups: s.groups }
  }
}

impl MongoPool {
  fn get_snapshot_models(&self) -> Collection<SnapshotModel> {
    self.default_database().unwrap().collection("snapshots")
  }
}
