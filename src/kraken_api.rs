use base64;
use hmac::{Hmac, Mac, NewMac};
use reqwest;
use reqwest::Client;
use serde::export::Formatter;
use serde::Deserialize;
use sha2::{Digest, Sha256, Sha512};
use std::env;
use std::{
    collections::HashMap,
    error, fmt,
    time::{self, Duration},
};
use url::{form_urlencoded, Url};

const ASSETS_URL: &'static str = "https://api.kraken.com/0/public/Assets";
const ASSET_PAIRS_URL: &'static str = "https://api.kraken.com/0/public/AssetPairs";
const TICKER_URL: &'static str = "https://api.kraken.com/0/public/Ticker";
const ORDER_BOOK_URL: &'static str = "https://api.kraken.com/0/public/Depth";
const ACCOUNT_BALANCE_URL: &'static str = "https://api.kraken.com/0/private/Balance";

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    Kraken(String),
    InvalidFormat,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Request(error) => write!(f, "{}", error),
            Self::InvalidFormat => write!(f, "Invalid format"),
            Self::Kraken(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for Errors {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Request(error) => error.source(),
            Self::InvalidFormat => None,
            Self::Kraken(_) => None,
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
    Ticker(HashMap<String, Ticker>),
    OrderBook(HashMap<String, OrderBook>),
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
pub struct Ticker {
    // Ask array (<price>, <whole lot volume>, <lot volume>)
    a: Vec<String>,
    // Bid array (<price>, <whole lot volume>, <lot volume>)
    b: Vec<String>,
    // Last trade closed array (<price>, <lot volume>)
    c: Vec<String>,
    // Volume array (<today>, <last 24 hours>)
    v: Vec<String>,
    // Volume weighted average price array (<today>, <last 24 hours>)
    p: Vec<String>,
    // Number of trades array (<today>, <last 24 hours>)
    t: Vec<u64>,
    // Low array(<today>, <last 24 hours>)
    l: Vec<String>,
    // High array(<today>, <last 24 hours>)
    h: Vec<String>,
    // Today's opening price
    o: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub asks: Vec<(String, String, u64)>,
    pub bids: Vec<(String, String, u64)>,
}

pub struct Kraken {
    credentials: Credentials,
    client: Client,
}

// TODO add private methods:
//  * account balance
//  * trade balance
//  * open orders
//  * closed orders
//  * orders info
//  * trades history
//  * open positions
//  * ledgers info
//  * ledgers
//  * trade volume
//  * add order
//  * cancel order
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
        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::Assets(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub async fn asset_pairs(&self, params: Option<HashMap<&str, &str>>) -> Result<AssetPairs, Errors> {
        let mut request = self.client.get(ASSET_PAIRS_URL);

        if let Some(params) = params {
            let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
            request = request.query(&query_params);
        }

        let response = request.send().await?.json::<KrakenResponse>().await?;
        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::AssetPairs(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub async fn ticker(&self, params: HashMap<&str, &str>) -> Result<HashMap<String, Ticker>, Errors> {
        let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
        let request = self.client.get(TICKER_URL).query(&query_params);

        let response = request.send().await?.json::<KrakenResponse>().await?;
        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::Ticker(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub async fn order_book(&self, params: HashMap<&str, &str>) -> Result<HashMap<String, OrderBook>, Errors> {
        let query_params: Vec<(&str, &str)> = params.iter().map(|(key, value)| (*key, *value)).collect();
        let request = self.client.get(ORDER_BOOK_URL).query(&query_params);

        let response = request.send().await?.json::<KrakenResponse>().await?;
        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::OrderBook(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub async fn account_balance(&self, params: Option<HashMap<&str, &str>>) -> Result<(), Errors> {
        let nonce = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        let mut query_params = HashMap::new();
        query_params.insert("nonce", nonce.as_str());

        // If nonce was passed in params, we overwrite out previously created one with it
        if let Some(params) = params {
            for (key, value) in params {
                query_params.insert(key, value);
            }
        }

        let message = "c29tZSBkYXRhIHdpdGggACBhbmQg77u/";
        // TODO create new error type for invalid base64 string
        let secret_base64 = base64::decode(&self.credentials.secret).unwrap();
        let url = Url::parse(ACCOUNT_BALANCE_URL).unwrap();
        let signature = create_signature(url.path(), query_params, &secret_base64);
        println!("sin: {}", signature);
        todo!();
    }
}

// Message signature using HMAC-SHA512 of (URI path + SHA256(nonce + POST data)) and base64 decoded secret API key
fn create_signature(url: &str, params: HashMap<&str, &str>, secret: &Vec<u8>) -> String {
    // We know it exists because we pass it in account_balance
    let nonce = params.get("nonce").unwrap();
    let url = Url::parse(url).unwrap();
    let url = url.path();

    let mut encoded_params = form_urlencoded::Serializer::new(String::new());
    for (key, value) in params.iter() {
        encoded_params.append_pair(key, value);
    }
    let encoded_params = encoded_params.finish();

    let mut hasher = Sha256::new();
    hasher.update(nonce.to_string() + &encoded_params);
    let sha = hasher.finalize();

    let buffer = [url.as_bytes(), sha.as_slice()].concat();
    let mut hasher: Hmac<Sha512> = Hmac::new_varkey(secret).unwrap();
    hasher.update(&buffer);

    base64::encode(hasher.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_signature() {
        let url = ACCOUNT_BALANCE_URL;
        let secret = env::var("KRAKEN_SECRET_KEY").expect("KRAKEN_API_KEY not found in env");
        let secret64 = base64::decode(secret).unwrap();
        let timestamps = vec![
            "1603301840074000",
            "1603301843686000",
            "1603301937946000",
            "1603293009951000",
        ];
        let hashes = vec![
            "cPLxlJ9pzq7xwPaERVJKFhPcf4uSjsScV7ms9TN6YS9sTmyC5BQUwaJVOMD7my3GIdstamo7G4VVOVp3si75AA==",
            "CXIm1rRjBTEDtPPslxf/Ll6pfjI9aPNFGbA3DLWm/yqXhC5e5SjNlSFjkKNSmDkOYu5Jtm1/lBixiQ0VGgGzog==",
            "2PLFGUf6Xyb3wC16FjpI4iTIxmmIBsf54sMg4IhDHefkHia6FH3PGtnjJEuJq5zCEH32jPuJapAyuzeRAomHvg==",
            "fnyLS6vcz223yuXWR0rzlKteEgSCjIvkL2P8KiekpwcisBxYZKCrrovmN0AW65UwM51HQJkNydbErq1CdZ+3iw==",
        ];

        for (i, nonce) in timestamps.iter().enumerate() {
            let params: HashMap<&str, &str> = vec![("nonce", *nonce)].into_iter().collect();
            let expected = hashes.get(i).unwrap();
            let signature = create_signature(url, params, &secret64);
            assert_eq!(*expected, signature.as_str());
        }
    }
}
