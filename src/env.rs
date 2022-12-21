use std::str::FromStr;

type EnvParam = &'static str;

pub const DB_URL: EnvParam = "DATABASE_CONNECTION";

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
  dotenvy::var(var).ok().and_then(|x| x.parse().ok())
}

pub fn check<T: FromStr>(var: &'static str) -> bool {
  match parse_var::<T>(var) {
    Some(_) => true,
    None => {
      error!("Var {}: {} is not present", var, std::any::type_name::<T>().split("::").last().unwrap());
      false
    }
  }
}

pub fn check_env_vars() {
  info!("Validating .env vars");
  let mut failed = false;

  failed |= !check::<String>(DB_URL);

  failed.then(|| panic!("Not all environment args are set"));
}
