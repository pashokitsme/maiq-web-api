#[macro_use]
extern crate rocket;

#[macro_use]
extern crate tracing;

mod api;
mod db;
mod env;

use api::{error::not_found, routes::*};
use rocket::Error;

#[rocket::main]
async fn main() -> Result<(), Error> {
  // db::init(&pool).await.map_err(CustomError::new)?;
  tracing_subscriber::fmt::init();
  env::check_env_vars();
  let pool = db::init().await.expect("Error while connecting to database");
  let names = pool.list_database_names(None, None).await.unwrap();
  info!("{:?}", names);
  _ = rocket::build()
    .register("/", catchers![not_found])
    .mount("/", routes![index])
    .mount("/api", routes![index, naive, update, today])
    .manage(pool)
    .launch()
    .await?;
  Ok(())
}
