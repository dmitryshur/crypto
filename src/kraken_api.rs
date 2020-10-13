use reqwest;
use reqwest::Client;
use serde::export::Formatter;
use serde::Deserialize;
use std::{collections::HashMap, error, fmt, time::Duration};

const ASSETS_URL: &'static str = "https://api.kraken.com/0/public/Assets";
const ASSET_PAIRS_URL: &'static str = "https://api.kraken.com/0/public/AssetPairs";
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
    Assets(HashMap<String, Asset>),
    AssetPairs(AssetPairs),
    Ticker(TickerResponse),
    OrderBook(OrderBookResponse),
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    aclass: String,
    altname: String,
    decimals: u64,
    display_decimals: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AssetPairs {
    Info(HashMap<String, AssetPairInfo>),
    Fees(HashMap<String, AssetPairFees>),
    Margin(HashMap<String, AssetPairMargin>),
}

#[derive(Debug, Deserialize)]
pub struct AssetPairInfo {
    altname: String,
    wsname: Option<String>,
    aclass_base: String,
    base: String,
    aclass_quote: String,
    quote: String,
    lot: String,
    pair_decimals: u64,
    lot_decimals: u64,
    lot_multiplier: u64,
    leverage_buy: Vec<u64>,
    leverage_sell: Vec<u64>,
    fees: Vec<Vec<f64>>,
    fees_maker: Option<Vec<Vec<f64>>>,
    fee_volume_currency: String,
    margin_call: u64,
    margin_stop: u64,
    ordermin: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssetPairFees {
    fees: Vec<Vec<f64>>,
    fee_volume_currency: String,
}

#[derive(Debug, Deserialize)]
pub struct AssetPairMargin {
    margin_call: u64,
    margin_level: u64,
}

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
//  fix warnings
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

    // TODO refactor. send api request via a separate method
    pub async fn assets(&self, params: Option<HashMap<&str, &str>>) -> Result<HashMap<String, Asset>, Errors> {
        let mut request = self.client.get(ASSETS_URL);

        if let Some(params) = params {
            let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
            request = request.query(&query_params);
        }

        let response = request.send().await?.json::<KrakenResponse>().await?;

        // TODO this is wrong. test with a request which returns a definite error, like when forgetting
        //  to specify a pair in Ticker
        match response.result.unwrap() {
            Responses::Assets(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    // TODO something is wrong when no params are provided
    pub async fn asset_pairs(&self, params: Option<HashMap<&str, &str>>) -> Result<AssetPairs, Errors> {
        let mut request = self.client.get(ASSET_PAIRS_URL);

        if let Some(params) = params {
            let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
            request = request.query(&query_params);
        }

        let response = request.send().await?.json::<KrakenResponse>().await?;

        // TODO this is wrong. test with a request which returns a definite error, like when forgetting
        //  to specify a pair in Ticker
        match response.result.unwrap() {
            Responses::AssetPairs(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }
}
