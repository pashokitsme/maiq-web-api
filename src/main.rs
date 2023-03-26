#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

mod api;
mod env;
mod storage;

use std::{env::args, fs, sync::Arc};

use api::{
  error::{internal_server_error, not_found, unauthorized},
  routes::*,
};

use chrono::NaiveTime;

use maiq_parser::utils;
use rocket::{
  fairing::{Fairing, Info, Kind},
  http::Header,
  Request, Response,
};

use storage::{cache::CachePool, mongo::MongoPool};
use tokio::sync::RwLock;

#[rocket::main]
async fn main() {
  dotenvy::dotenv().ok();
  pretty_env_logger::init();
  env::check_env_vars();
  {
    use std::path::PathBuf;
    info!("Path: {}", args().next().unwrap());
    let dirs = fs::read_dir("./")
      .unwrap()
      .map(|d| d.unwrap().path())
      .collect::<Vec<PathBuf>>();
    info!("Folders: {:?}", dirs);
  }
  maiq_parser::warmup_defaults();

  let mongo = MongoPool::init().await.expect("Error while connecting to database");
  let cache = CachePool::new(mongo.clone()).await;
  cache.write().await.update_tick().await;

  startup_cache_updater(cache.clone());

  _ = rocket::build()
    .register("/", catchers![not_found, internal_server_error, unauthorized])
    .mount("/", routes![index])
    .mount("/api", routes![index, latest, latest_group, poll, snapshot_by_date, snapshot_by_id, default])
    .mount("/api/dev", routes![cached])
    .attach(Cors)
    .manage(mongo)
    .manage(cache)
    .launch()
    .await
    .expect("Error while running Rocket");
}

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
  fn info(&self) -> Info {
    Info { name: "CORS", kind: Kind::Response }
  }

  async fn on_response<'r>(&self, _: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new("Access-Control-Allow-Methods", "GET"));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

pub fn startup_cache_updater(cache: Arc<RwLock<CachePool>>) {
  tokio::spawn(async move {
    let cache = cache.clone();
    let cache_ref = cache.clone();
    tokio::spawn(async move {
      loop {
        let now = utils::time::now().time();
        let wait_s = NaiveTime::from_hms_opt(23, 59, 59)
          .unwrap()
          .signed_duration_since(now)
          .num_seconds()
          + 1;

        info!("Waiting for {}s to drop previous day poll", wait_s);
        tokio::time::sleep(std::time::Duration::from_secs(wait_s as u64)).await;
        cache_ref.write().await.reset();
      }
    });

    loop {
      let cache_ref = cache.clone();

      _ = tokio::spawn(async move {
        let mut interval = storage::cache::interval();
        interval.tick().await;
        loop {
          info!("Sleeping for {:?}", interval.period());
          interval.tick().await;
          cache_ref.write().await.update_tick().await;
        }
      })
      .await;
      error!("Seems snapshot updater is panicked. Restarting thread in 10s!");
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
  });
}
