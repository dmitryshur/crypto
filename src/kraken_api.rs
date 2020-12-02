use base64;
use hmac::{Hmac, Mac, NewMac};
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
    Client, RequestBuilder,
};
use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};
use serde_json::Value;
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::HashMap,
    error, fmt,
    str::FromStr,
    time::{self, Duration},
};
use url::{form_urlencoded, Url};

pub struct Urls {
    assets: String,
    asset_pairs: String,
    ticker: String,
    order_book: String,
    account_balance: String,
    trade_balance: String,
    open_orders: String,
}

impl Urls {
    pub fn new(domain: &str) -> Self {
        Self {
            assets: format!("{}{}", domain, "/0/public/Assets"),
            asset_pairs: format!("{}{}", domain, "/0/public/AssetPairs"),
            ticker: format!("{}{}", domain, "/0/public/Ticker"),
            order_book: format!("{}{}", domain, "/0/public/Depth"),
            account_balance: format!("{}{}", domain, "/0/private/Balance"),
            trade_balance: format!("{}{}", domain, "/0/private/TradeBalance"),
            open_orders: format!("{}{}", domain, "/0/private/OpenOrders"),
        }
    }
}

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

fn from_f64_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    f64::from_str(s).map_err(D::Error::custom)
}

fn from_f64_option_str<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<&str> = Deserialize::deserialize(deserializer)?;

    return match s {
        Some(s) => {
            println!("s: {:?}", s);
            let num: f64 = s.parse().unwrap();
            println!("num: {:?}", num);
            Ok(Some(num))
        }
        None => {
            println!("none");
            Ok(None)
        }
    };
}

fn from_f64_str_vec<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Deserialize::deserialize(deserializer)?;
    let floats_vec: Vec<f64> = s.iter().map(|num| num.parse().unwrap()).collect();

    Ok(floats_vec)
}

fn from_tuple<'de, D>(deserializer: D) -> Result<Vec<(f64, f64, u64)>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<(String, String, u64)> = Deserialize::deserialize(deserializer)?;
    let floats_vec: Vec<(f64, f64, u64)> = s
        .iter()
        .map(|tuple| (tuple.0.parse().unwrap(), tuple.1.parse().unwrap(), tuple.2))
        .collect();

    Ok(floats_vec)
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
    TradeBalance(TradeBalance),
    // TODO convert to float
    Balance(HashMap<String, String>),
    OpenOrder { open: HashMap<String, OpenOrder> },
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
    #[serde(default)]
    #[serde(deserialize_with = "from_f64_option_str")]
    ordermin: Option<f64>,
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
    #[serde(deserialize_with = "from_f64_str_vec")]
    a: Vec<f64>,
    // Bid array (<price>, <whole lot volume>, <lot volume>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    b: Vec<f64>,
    // Last trade closed array (<price>, <lot volume>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    c: Vec<f64>,
    // Volume array (<today>, <last 24 hours>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    v: Vec<f64>,
    // Volume weighted average price array (<today>, <last 24 hours>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    p: Vec<f64>,
    // Number of trades array (<today>, <last 24 hours>)
    t: Vec<u64>,
    // Low array(<today>, <last 24 hours>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    l: Vec<f64>,
    // High array(<today>, <last 24 hours>)
    #[serde(deserialize_with = "from_f64_str_vec")]
    h: Vec<f64>,
    // Today's opening price
    #[serde(deserialize_with = "from_f64_str")]
    o: f64,
}

#[derive(Debug, Deserialize)]
pub struct OrderBook {
    #[serde(deserialize_with = "from_tuple")]
    pub asks: Vec<(f64, f64, u64)>,
    #[serde(deserialize_with = "from_tuple")]
    pub bids: Vec<(f64, f64, u64)>,
}

#[derive(Debug, Deserialize)]
pub struct TradeBalance {
    // Equivalent balance (combined balance of all currencies)
    #[serde(deserialize_with = "from_f64_str")]
    pub eb: f64,
    // Trade balance (combined balance of all equity currencies)
    #[serde(deserialize_with = "from_f64_str")]
    pub tb: f64,
    // Margin amount of open positions
    #[serde(deserialize_with = "from_f64_str")]
    pub m: f64,
    // Unrealized net profit/loss of open positions
    #[serde(deserialize_with = "from_f64_str")]
    pub n: f64,
    // Cost basis of open positions
    #[serde(deserialize_with = "from_f64_str")]
    pub c: f64,
    // Current floating valuation of open positions
    #[serde(deserialize_with = "from_f64_str")]
    pub v: f64,
    // Equity = trade balance + unrealized net profit/loss
    #[serde(deserialize_with = "from_f64_str")]
    pub e: f64,
    // Free margin = equity - initial margin (maximum margin available to open new positions)
    #[serde(deserialize_with = "from_f64_str")]
    pub mf: f64,
    // Margin level = (equity / initial margin) * 100
    #[serde(default)]
    #[serde(deserialize_with = "from_f64_option_str")]
    pub ml: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrder {
    // Referral order transaction id that created this order
    refid: Option<String>,
    // User reference id
    userref: u64,
    // Status of order:
    //     pending = order pending book entry
    //     open = open order
    //     closed = closed order
    //     canceled = order canceled
    //     expired = order expired
    status: String,
    // Unix timestamp of when order was placed
    opentm: f64,
    // Unix timestamp of order start time (or 0 if not set)
    starttm: f64,
    // Unix timestamp of order end time (or 0 if not set)
    expiretm: f64,
    descr: OpenOrderDescription,
    // Volume of order (base currency unless viqc set in oflags)
    #[serde(deserialize_with = "from_f64_str")]
    vol: f64,
    // Volume executed (base currency unless viqc set in oflags)
    #[serde(deserialize_with = "from_f64_str")]
    vol_exec: f64,
    // Total cost (quote currency unless unless viqc set in oflags)
    #[serde(deserialize_with = "from_f64_str")]
    cost: f64,
    // Total fee (quote currency)
    #[serde(deserialize_with = "from_f64_str")]
    fee: f64,
    // Average price (quote currency unless viqc set in oflags)
    #[serde(deserialize_with = "from_f64_str")]
    price: f64,
    // Stop price (quote currency, for trailing stops)
    #[serde(deserialize_with = "from_f64_str")]
    stopprice: f64,
    // Triggered limit price (quote currency, when limit based order type triggered)
    #[serde(deserialize_with = "from_f64_str")]
    limitprice: f64,
    // Comma delimited list of miscellaneous info
    //     stopped = triggered by stop price
    //     touched = triggered by touch price
    //     liquidated = liquidation
    //     partial = partial fill
    misc: String,
    // Comma delimited list of order flags
    //     viqc = volume in quote currency
    //     fcib = prefer fee in base currency (default if selling)
    //     fciq = prefer fee in quote currency (default if buying)
    //     nompp = no market price protection
    oflags: String,
    // Array of trade ids related to order (if trades info requested and data available)
    trades: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrderDescription {
    // Asset pair
    pair: String,
    // Type of order
    #[serde(rename = "type")]
    kind: String,
    // Order type:
    //     market
    //     limit (price = limit price)
    //     stop-loss (price = stop loss price)
    //     take-profit (price = take profit price)
    //     stop-loss-profit (price = stop loss price, price2 = take profit price)
    //     stop-loss-profit-limit (price = stop loss price, price2 = take profit price)
    //     stop-loss-limit (price = stop loss trigger price, price2 = triggered limit price)
    //     take-profit-limit (price = take profit trigger price, price2 = triggered limit price)
    //     trailing-stop (price = trailing stop offset)
    //     trailing-stop-limit (price = trailing stop offset, price2 = triggered limit offset)
    //     stop-loss-and-limit (price = stop loss price, price2 = limit price)
    //     settle-position
    ordertype: String,
    // Primary price
    #[serde(deserialize_with = "from_f64_str")]
    price: f64,
    // Secondary price
    #[serde(deserialize_with = "from_f64_str")]
    price2: f64,
    // This could be a number if enabled in the account
    leverage: String,
    // Order description
    order: String,
    // Conditional close order description (if conditional close set)
    close: String,
}

pub struct Kraken {
    credentials: Credentials,
    client: Client,
    urls: Urls,
}

// TODO add private methods:
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
//  Maybe change the naming of the params returned from kraken
impl Kraken {
    pub fn new(credentials: Credentials, urls: Urls) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Can't create reqwest client");

        Self {
            credentials,
            client,
            urls,
        }
    }

    pub async fn assets(&self, params: &[(&str, &str)]) -> Result<HashMap<String, Asset>, Errors> {
        let request = self.client.get(&self.urls.assets).query(params);
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
        let request = self.client.get(&self.urls.asset_pairs).query(params);
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
        let request = self.client.get(&self.urls.ticker).query(params);
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
        let request = self.client.get(&self.urls.order_book).query(params);
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

    pub async fn account_balance(&self, params: &[(&str, &str)]) -> Result<HashMap<String, String>, Errors> {
        let request = self.private_request(&self.urls.account_balance, params)?;
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

    pub async fn trade_balance(&self, params: &[(&str, &str)]) -> Result<TradeBalance, Errors> {
        let request = self.private_request(&self.urls.trade_balance, params)?;
        let response = request.send().await?.json::<KrakenResponse>().await?;

        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::TradeBalance(response) => Ok(response),
            _ => Err(Errors::InvalidFormat),
        }
    }

    pub async fn open_orders(&self, params: &[(&str, &str)]) -> Result<HashMap<String, OpenOrder>, Errors> {
        let request = self.private_request(&self.urls.open_orders, params)?;
        let response = request.send().await?.json::<KrakenResponse>().await?;

        if response.error.len() != 0 {
            let error = response.error.join(" ");
            return Err(Errors::Kraken(error));
        }

        match response.result.unwrap() {
            Responses::OpenOrder { open } => Ok(open),
            _ => Err(Errors::InvalidFormat),
        }
    }

    // TODO replace url type with IntoUrl
    fn private_request(&self, url: &str, params: &[(&str, &str)]) -> Result<RequestBuilder, Errors> {
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

        let signature = create_signature(url, query_params, &self.credentials.secret)?;

        let mut headers = HeaderMap::new();
        headers.insert("API-Key", HeaderValue::from_str(&self.credentials.api_key).unwrap());
        headers.insert("API-Sign", HeaderValue::from_str(&signature).unwrap());
        let post_data = PrivatePostData { nonce, otp: None };

        Ok(self.client.post(url).headers(headers).form(&post_data))
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
        if *key == "nonce" || *key == "otp" {
            encoded_params.append_pair(key, value);
        }
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

    #[test]
    fn test_create_signature() {
        let url = "https://api.kraken.com/0/private/Balance";
        let secret = "NZTRqjFqtb7Jbg5Yx7iRelcfCxiB7pL1FvvK3tokScThZDl0z7oi/m5aHhtKcUp2dIpT8qIbaMfp01Glzw24Ag==";
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
            let signature = create_signature(url, params, secret);
            assert_eq!(signature.is_ok(), true);
            assert_eq!(*expected, signature.unwrap().as_str());
        }
    }
}
