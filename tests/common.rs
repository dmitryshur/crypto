extern crate kraken_api;

use kraken_api::Credentials;
use std::env;

pub fn create_credentials() -> Credentials {
    let key = env::var("KRAKEN_API_KEY").expect("KRAKEN_API_KEY not found in env");
    let secret = env::var("KRAKEN_SECRET_KEY").expect("KRAKEN_SECRET_KEY not found in env");

    Credentials::new(key, secret)
}
