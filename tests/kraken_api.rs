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
    assert_eq!(
        response.unwrap().len() > 0,
        true,
        "Testing if there are any assets present in the assets response"
    );

    let mut params = HashMap::new();
    params.insert("asset", "algo,ada");

    // Should return only the requested ALGO and ADA assets
    let response = kraken.assets(Some(params)).await;

    assert_eq!(response.is_ok(), true);
    let response = response.unwrap();

    assert_eq!(
        response.len(),
        2,
        "Testing if the amount of assets in the response is equal to the amount of assets in the request params"
    );
    assert_eq!(
        response.contains_key("ALGO"),
        true,
        "Testing if ALGO asset is present in assets response"
    );
    assert_eq!(
        response.contains_key("ADA"),
        true,
        "Testing if ADA asset is present in assets response"
    );
}

#[tokio::test]
async fn asset_pairs_api() {
    let kraken = Kraken::new(common::create_credentials());

    let response = kraken.asset_pairs(None).await;
    assert_eq!(response.is_ok(), true);

    match response.unwrap() {
        AssetPairs::Info(pairs) => {
            assert_eq!(
                pairs.len() > 0,
                true,
                "Testing if there are any asset pairs present in the asset pairs response"
            );
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
            assert_eq!(
                pairs.len() == 1,
                true,
                "Testing if the number of pairs is 1 in request for fees"
            );
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
            assert_eq!(
                pairs.len() == 2,
                true,
                "Testing if the number of pairs is 2 in request for margin"
            );
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with pair and margin params");
        }
    }
}
