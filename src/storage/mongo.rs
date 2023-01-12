use std::ops::Deref;

use maiq_parser::{utils, Fetch, Snapshot};
use mongodb::bson::DateTime;
use mongodb::options::ClientOptions;
use mongodb::{bson::doc, options::FindOptions};

use crate::env;
use crate::{api::error::ApiError, storage::SnapshotModel};

use super::SnapshotPool;

pub type MongoClient = mongodb::Client;
pub type MongoError = mongodb::error::Error;

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

impl MongoPool {
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

  pub async fn get_latest_today(&self) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let today = DateTime::from_chrono(utils::now_date(0));
    let opts = FindOptions::builder()
      .sort(doc! { "parsed_date": 1, "date": 1 })
      .limit(1)
      .build();
    let mut cur = snapshots.find(doc! { "date": today }, opts).await?;
    if !cur.advance().await? {
      warn!("There is no snapshots for today");
      return Ok(None);
    };

    Ok(Some(cur.deserialize_current()?.into()))
  }

  pub async fn get_latest_next(&self) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let time = DateTime::from_chrono(utils::now_date(1));
    let opts = FindOptions::builder()
      .sort(doc! { "parsed_date": 1, "date": 1 })
      .limit(1)
      .build();
    let mut cur = snapshots.find(doc! { "date": { "$gte": time } }, opts).await?;
    if !cur.advance().await? {
      warn!("There is no snapshots for next day");
      return Ok(None);
    }

    Ok(Some(cur.deserialize_current()?.into()))
  }
}

#[rocket::async_trait]
impl SnapshotPool for MongoPool {
  async fn save(&mut self, snapshot: &Snapshot) -> Result<(), ApiError> {
    let snapshots = self.get_snapshot_models();
    let mut cur = snapshots.find(doc! { "uid": &snapshot.uid }, None).await?;

    if cur.advance().await? == true {
      info!("Snapshot {} already exists", snapshot.uid);
      return Ok(());
    }

    info!("Saving new snapshot {}", snapshot.uid);
    let snapshot_internal = SnapshotModel::from(snapshot);
    snapshots.insert_one(snapshot_internal, None).await?;
    Ok(())
  }

  async fn latest(&self, mode: Fetch) -> Result<Option<Snapshot>, ApiError> {
    match mode {
      Fetch::Today => self.get_latest_today().await.map_err(Into::into),
      Fetch::Next => self.get_latest_next().await.map_err(Into::into),
    }
  }

  async fn by_uid<T: AsRef<str> + Send>(&self, uid: T) -> Result<Option<Snapshot>, ApiError> {
    let snapshots = self.get_snapshot_models();
    let mut cur = snapshots.find(doc! { "uid": uid.as_ref() }, None).await?;
    if !cur.advance().await? {
      warn!("Snapshot {} not found", uid.as_ref());
      return Ok(None);
    }

    Ok(Some(cur.deserialize_current()?.into()))
  }
}
