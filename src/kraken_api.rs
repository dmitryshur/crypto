use reqwest;
use reqwest::{Client, Error};
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::time;
use tokio;
use tokio::time::Duration;

const ASSETS_URL: &'static str = "https://api.kraken.com/0/public/Assets";
const ASSET_PAIR_URL: &'static str = "https://api.kraken.com/0/public/AssetPairs";
const TICKER_URL: &'static str = "https://api.kraken.com/0/public/Ticker";
const ORDER_BOOK_URL: &'static str = "https://api.kraken.com/0/public/Depth";

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    InvalidFormat,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Request(error) => write!(f, "{}", error),
            Self::InvalidFormat => write!(f, "Invalid format"),
        }
    }
}

impl error::Error for Errors {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Request(error) => error.source(),
            Self::InvalidFormat => None,
        }
    }
}

impl From<reqwest::Error> for Errors {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

pub struct Credentials {
    api_key: String,
    secret: String,
}

impl Credentials {
    pub fn new(api_key: String, secret: String) -> Self {
        Self { api_key, secret }
    }
}

#[derive(Debug, Deserialize)]
struct KrakenResponse {
    error: Vec<String>,
    result: Option<Responses>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Responses {
    Assets(AssetsResponse),
    AssetPairs(AssetPairsResponse),
    Ticker(TickerResponse),
    OrderBook(OrderBookResponse),
}

#[derive(Debug, Deserialize)]
struct AssetsResponse(HashMap<String, Asset>);

#[derive(Debug, Deserialize)]
pub struct Asset {
    aclass: String,
    altname: String,
    decimals: u8,
    display_decimals: u8,
}

#[derive(Debug, Deserialize)]
struct AssetPairsResponse {}

#[derive(Debug, Deserialize)]
struct TickerResponse {}

#[derive(Debug, Deserialize)]
struct OrderBookResponse {}

pub struct Kraken {
    credentials: Credentials,
    client: Client,
}

// TODO implement methods for the following requests:
//  * AssetPairs
//  * Ticker
//  * Depth (order book)
impl Kraken {
    pub fn new(credentials: Credentials) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Can't create reqwest client");

        Self { credentials, client }
    }

    pub async fn assets(&self, params: Option<HashMap<&str, &str>>) -> Result<HashMap<String, Asset>, Errors> {
        let mut request = self.client.get(ASSETS_URL);

        // Add additional request params if present
        if let Some(params) = params {
            let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
            request = request.query(&query_params);
        }

        let response = request.send().await?.json::<KrakenResponse>().await?;

        // TODO this is wrong. test with a request which returns a definite error, like when forgetting
        //  to specify a pair in Ticker
        match response.result.unwrap() {
            Responses::Assets(response) => Ok(response.0),
            _ => Err(Errors::InvalidFormat),
        }
    }
}
