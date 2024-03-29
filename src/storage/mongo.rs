use std::ops::Deref;

use maiq_parser::{utils::time::*, Fetch, Snapshot};
use mongodb::bson::doc;
use mongodb::bson::DateTime;
use mongodb::options::{ClientOptions, FindOneAndReplaceOptions, FindOneOptions};

use crate::env;
use crate::{api::error::ApiError, storage::SnapshotModel};

use super::SnapshotPool;

pub type MongoClient = mongodb::Client;
pub type MongoError = mongodb::error::Error;

#[derive(Clone)]
pub struct MongoPool(MongoClient);

impl Deref for MongoPool {
  type Target = MongoClient;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl MongoPool {
  pub async fn init() -> Result<MongoPool, MongoError> {
    info!("Connecting to database..");

    let mut opts = ClientOptions::parse(env::db_url()).await?;
    opts.app_name = Some("maiq-web".into());
    opts.default_database = Some(env::db_default_collection());

    Ok(MongoPool(MongoClient::with_options(opts)?))
  }

  async fn get_latest_today(&self) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let today = DateTime::from_chrono(now_date());
    let opts = FindOneOptions::builder().sort(doc! { "parsed_date": -1 }).build();
    let res = snapshots
      .find_one(doc! { "date": today }, opts)
      .await?
      .and_then(Into::into);
    Ok(res)
  }

  async fn get_latest_next(&self) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let time = DateTime::from_chrono(now_date_offset(1));
    let opts = FindOneOptions::builder().sort(doc! { "parsed_date": -1 }).build();
    let res = snapshots
      .find_one(doc! { "date": { "$gte": time } }, opts)
      .await?
      .and_then(Into::into);
    Ok(res)
  }

  pub async fn by_date(&self, date: DateTime) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let opts = FindOneOptions::builder().sort(doc! { "parsed_date": -1 }).build();
    let res = snapshots
      .find_one(doc! { "date": date }, opts)
      .await?
      .and_then(Into::into);
    Ok(res)
  }
}

#[rocket::async_trait]
impl SnapshotPool for MongoPool {
  async fn save(&mut self, snapshot: &Snapshot) -> Result<(), ApiError> {
    let snapshots = self.get_snapshot_models();
    let model = SnapshotModel::from(snapshot);
    let opts = FindOneAndReplaceOptions::builder().upsert(true).build();
    snapshots
      .find_one_and_replace(doc! { "date": model.date }, model, opts)
      .await
      .unwrap();

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
