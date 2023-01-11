use super::{MongoError, MongoPool};
use maiq_parser::{utils, Fetch, Snapshot};
use mongodb::{
  bson::{doc, DateTime},
  options::FindOptions,
};

impl MongoPool {
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

  pub async fn get_latest(&self, fetch: Fetch) -> Result<Option<Snapshot>, MongoError> {
    match fetch {
      Fetch::Today => self.get_latest_today().await,
      Fetch::Tomorrow => self.get_latest_next().await,
    }
  }

  pub async fn get_by_uid<'a>(&self, uid: &'a str) -> Result<Option<Snapshot>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let mut cur = snapshots.find(doc! { "uid": uid }, None).await?;
    if !cur.advance().await? {
      warn!("Snapshot {} not found", uid);
      return Ok(None);
    }

    Ok(Some(cur.deserialize_current()?.into()))
  }
}
