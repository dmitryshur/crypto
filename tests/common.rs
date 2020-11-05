extern crate kraken_api;

use kraken_api::{Credentials, Urls};
use std::env;

pub fn create_credentials() -> Credentials {
    let key = env::var("KRAKEN_API_KEY").expect("KRAKEN_API_KEY not found in env");
    let secret = env::var("KRAKEN_SECRET_KEY").expect("KRAKEN_SECRET_KEY not found in env");

    Credentials::new(key, secret)
}

pub fn create_urls() -> Urls {
    let domain = "http://0.0.0.0:4000";
    Urls::new(domain)
}
