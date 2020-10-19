extern crate kraken_api;

use kraken_api::{AssetPairs, Kraken};
use std::collections::HashMap;
use tokio;

mod common;

#[tokio::test]
async fn assets_api() {
    let kraken = Kraken::new(common::create_credentials());

    // Should return all the assets
    let response = kraken.assets(None).await;
    assert_eq!(response.is_ok(), true);
    assert_eq!(response.unwrap().len() > 0, true,);

    let mut params = HashMap::new();
    params.insert("asset", "algo,ada");

    // Should return only the requested ALGO and ADA assets
    let response = kraken.assets(Some(params)).await;

    assert_eq!(response.is_ok(), true);
    let response = response.unwrap();

    assert_eq!(response.len(), 2,);
    assert_eq!(response.contains_key("ALGO"), true,);
    assert_eq!(response.contains_key("ADA"), true,);
}

#[tokio::test]
async fn asset_pairs_api() {
    let kraken = Kraken::new(common::create_credentials());

    let response = kraken.asset_pairs(None).await;
    assert_eq!(response.is_ok(), true);

    match response.unwrap() {
        AssetPairs::Info(pairs) => {
            assert_eq!(pairs.len() > 0, true,);
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with no params");
        }
    }

    let mut params = HashMap::new();
    params.insert("pair", "XXRPZUSD");
    params.insert("info", "fees");

    let response = kraken.asset_pairs(Some(params)).await;
    assert_eq!(response.is_ok(), true);

    match response.unwrap() {
        AssetPairs::Fees(pairs) => {
            assert_eq!(pairs.len() == 1, true,);
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with pair and fees params");
        }
    }

    let mut params = HashMap::new();
    params.insert("pair", "XXRPZUSD,XETHXXBT.d");
    params.insert("info", "margin");

    let response = kraken.asset_pairs(Some(params)).await;
    assert_eq!(response.is_ok(), true);

    match response.unwrap() {
        AssetPairs::Margin(pairs) => {
            assert_eq!(pairs.len() == 2, true,);
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with pair and margin params");
        }
    }
}
#[tokio::test]
async fn ticker_api() {
    let kraken = Kraken::new(common::create_credentials());
    let mut params = HashMap::new();
    params.insert("pair", "XXRPZUSD,ADAETH");

    let response = kraken.ticker(params).await;
    assert_eq!(response.is_ok(), true);

    let response = response.unwrap();

    assert_eq!(response.len() == 2, true);
    assert_eq!(response.contains_key("XXRPZUSD"), true);
    assert_eq!(response.contains_key("ADAETH"), true);
}
