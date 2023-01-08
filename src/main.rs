#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

mod api;
mod cache;
mod db;
mod env;

use api::{
  error::{internal_server_error, not_found, unauthorized},
  routes::*,
};

use cache::CachePool;
use rocket::{
  fairing::{Fairing, Info, Kind},
  http::Header,
  Request, Response,
};

#[rocket::main]
async fn main() {
  dotenvy::dotenv().expect("Unable to init .env");
  pretty_env_logger::init();
  env::check_env_vars();
  maiq_parser::warmup_defaults();

  let mongo = db::init().await.expect("Error while connecting to database");
  let cache = CachePool::new(mongo.clone()).await;

  let cache_ref = cache.clone();
  let mut cache_interval = cache::get_interval_from_env();

  tokio::spawn(async move {
    cache_interval.tick().await;
    loop {
      cache_interval.tick().await;
      cache_ref.lock().await.update_tick().await;
    }
  });

  let mongo_ref = mongo.clone();
  let cache_ref = cache.clone();

  _ = rocket::build()
    .register("/", catchers![not_found, internal_server_error, unauthorized])
    .mount("/", routes![index])
    .mount("/api", routes![index, latest, latest_group, poll, snapshot_by_id])
    .mount("/api/dev", routes![cached])
    .attach(Cors)
    .manage(mongo_ref)
    .manage(cache_ref)
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
