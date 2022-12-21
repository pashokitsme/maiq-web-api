#[macro_use]
extern crate rocket;

#[macro_use]
extern crate tracing;

mod api;
mod cache;
mod db;
mod env;

use api::{
  error::{internal_server_error, not_found},
  routes::*,
};
use maiq_parser::Fetch;
use rocket::Error;
use std::time::Duration;
use tokio::time;

#[rocket::main]
async fn main() -> Result<(), Error> {
  // db::init(&pool).await.map_err(CustomError::new)?;
  tracing_subscriber::fmt::init();
  env::check_env_vars();
  let pool = db::init().await.expect("Error while connecting to database");
  let pool_state = pool.clone();

  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_secs(60 * 5));
    let mut interval_between = time::interval(Duration::from_secs(15));
    loop {
      interval.tick().await;
      _ = cache::update(&pool, Fetch::Today).await;
      interval_between.tick().await;
      _ = cache::update(&pool, Fetch::Tomorrow).await;
    }
  });

  _ = rocket::build()
    .register("/", catchers![not_found, internal_server_error])
    .mount("/", routes![index])
    .mount("/api", routes![index])
    .mount("/api/latest/", routes![today, next])
    .mount("/api/snapshot/", routes![snapshot_by_id])
    .mount("/api/dev/", routes![naive])
    .manage(pool_state)
    .launch()
    .await?;
  Ok(())
}
