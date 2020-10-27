use base64;
use hmac::{Hmac, Mac, NewMac};
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
    Client, RequestBuilder,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
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
    Decode(base64::DecodeError),
    InvalidFormat,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => write!(f, "{}", error),
            Self::InvalidFormat => write!(f, "Invalid format"),
            Self::Kraken(error) => write!(f, "{}", error),
            Self::Decode(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for Errors {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Request(error) => error.source(),
            Self::InvalidFormat => None,
            Self::Kraken(_) => None,
            Self::Decode(error) => error.source(),
        }
    }
}

impl From<reqwest::Error> for Errors {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

impl From<base64::DecodeError> for Errors {
    fn from(error: base64::DecodeError) -> Self {
        Self::Decode(error)
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

#[derive(Debug, Serialize)]
struct PrivatePostData {
    nonce: String,
    otp: Option<String>,
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
    Balance(HashMap<String, String>),
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

    pub async fn assets(&self, params: &[(&str, &str)]) -> Result<HashMap<String, Asset>, Errors> {
        let request = self.client.get(ASSETS_URL).query(params);
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

    pub async fn asset_pairs(&self, params: &[(&str, &str)]) -> Result<AssetPairs, Errors> {
        let request = self.client.get(ASSET_PAIRS_URL).query(params);
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

    pub async fn ticker(&self, params: &[(&str, &str)]) -> Result<HashMap<String, Ticker>, Errors> {
        let request = self.client.get(TICKER_URL).query(params);
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

    pub async fn order_book(&self, params: &[(&str, &str)]) -> Result<HashMap<String, OrderBook>, Errors> {
        let request = self.client.get(ORDER_BOOK_URL).query(params);
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

    // TODO move the auth logic to create_signature. create private/public api methods. in the private
    //  method, add the post data/headers logic. in the public method, add the existing boilerplate code
    //  from all the methods above
    pub async fn account_balance(&self, params: &[(&str, &str)]) -> Result<HashMap<String, String>, Errors> {
        let request = self.private_request(params)?;
        let response = request.send().await?.json::<KrakenResponse>().await?;

        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::Balance(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub fn private_request(&self, params: &[(&str, &str)]) -> Result<RequestBuilder, Errors> {
        let nonce = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        let mut query_params = HashMap::new();
        query_params.insert("nonce", nonce.as_str());

        // This overwrites the nonce above if it was passed in params
        for (key, value) in params {
            query_params.insert(key, *value);
        }

        let signature = create_signature(ACCOUNT_BALANCE_URL, query_params, &self.credentials.secret)?;

        let mut headers = HeaderMap::new();
        headers.insert("API-Key", HeaderValue::from_str(&self.credentials.api_key).unwrap());
        headers.insert("API-Sign", HeaderValue::from_str(&signature).unwrap());
        let post_data = PrivatePostData { nonce, otp: None };

        Ok(self.client.post(ACCOUNT_BALANCE_URL).headers(headers).form(&post_data))
    }
}

// Message signature using HMAC-SHA512 of (URI path + SHA256(nonce + POST data)) and base64 decoded secret API key
fn create_signature(url: &str, params: HashMap<&str, &str>, secret: &str) -> Result<String, Errors> {
    // We know for sure "nonce" exists because we pass it in each send_private method
    let nonce = params.get("nonce").unwrap();
    let secret64 = base64::decode(secret)?;

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
    let mut hasher: Hmac<Sha512> = Hmac::new_varkey(&secret64).unwrap();
    hasher.update(&buffer);

    Ok(base64::encode(hasher.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // FIXME this will break each time the API key/secret is changed
    #[test]
    fn test_create_signature() {
        let url = ACCOUNT_BALANCE_URL;
        let secret = env::var("KRAKEN_SECRET_KEY").expect("KRAKEN_API_KEY not found in env");
        let timestamps = vec![
            "1603733933254000",
            "1603733979214000",
            "1603733998096000",
            "1603734014787000",
            "1603734032479000",
        ];
        let hashes = vec![
            "EAIoQ9XIntOdpiFHnt0UTmwqYZAmeeYFR/KwrhRqR1O6dLLNsT8I0R2GJ2p7M9OICQAol6kL9RF49l/aJaOKKw==",
            "vGNimOwAZNm31kjoB1CVh+vUbzj9PA68EYC/J/3zIo3129NwCzImNw6JmVwzALHjDwNR5w8VppatbLGfN5j7ow==",
            "isPAXtmUpxtSRWovhTc3G9qL1ZCeIi+LoyoezQBJbD/gjz4dZHhvvhv4oFtl5wPv7JhomU0TX6qpLborI612hw==",
            "SYC4C1aEQRtzI50AO4RqbX7LEINk8X4LYxviXD3LiDwnd5Gcg0n4mpoi7NjPsRM9r4kgF5JDq0150OXDd3v34g==",
            "HfK1Xpj66stc4uJOGnFJHPW7474h815EQ2DFCUmxZTykYGRAss07SLXB0o3T0/+3hswsymndXMTZeJURYHo5yA==",
        ];

        for (i, nonce) in timestamps.iter().enumerate() {
            let params: HashMap<&str, &str> = vec![("nonce", *nonce)].into_iter().collect();
            let expected = hashes.get(i).unwrap();
            let signature = create_signature(url, params, secret.as_str());
            assert_eq!(signature.is_ok(), true);
            assert_eq!(*expected, signature.unwrap().as_str());
        }
    }
}
