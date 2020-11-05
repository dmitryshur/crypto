extern crate kraken_api;

use kraken_api::{AssetPairs, Kraken};
use tokio;

mod common;

use common::{create_credentials, create_urls};

#[tokio::test]
async fn assets_api() {
    let kraken = Kraken::new(create_credentials(), create_urls());

    // Should return all the assets
    let response = kraken.assets(&[]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);
    assert_eq!(response.unwrap().len() > 0, true);

    // Should return only the requested ALGO and ADA assets
    let response = kraken.assets(&[("asset", "algo,ada")]).await;

    assert_eq!(response.is_ok(), true, "Response: {:?}", response);
    let response = response.unwrap();

    assert_eq!(response.len(), 2);
    assert_eq!(response.contains_key("ALGO"), true);
    assert_eq!(response.contains_key("ADA"), true);
}

#[tokio::test]
async fn asset_pairs_api() {
    let kraken = Kraken::new(create_credentials(), create_urls());

    let response = kraken.asset_pairs(&[]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    match response.unwrap() {
        AssetPairs::Info(pairs) => {
            assert_eq!(pairs.len() > 0, true,);
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with no params");
        }
    }

    let response = kraken.asset_pairs(&[("pair", "XXRPZUSD"), ("info", "fees")]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    match response.unwrap() {
        AssetPairs::Fees(pairs) => {
            assert_eq!(pairs.len() == 1, true,);
        }
        _ => {
            panic!("Invalid response from asset_pairs_api with pair and fees params");
        }
    }

    let response = kraken
        .asset_pairs(&[("pair", "XXRPZUSD,XETHXXBT.d"), ("info", "margin")])
        .await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

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
    let kraken = Kraken::new(create_credentials(), create_urls());

    let response = kraken.ticker(&[("pair", "XXRPZUSD,ADAETH")]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();

    assert_eq!(response.len() == 2, true);
    assert_eq!(response.contains_key("XXRPZUSD"), true);
    assert_eq!(response.contains_key("ADAETH"), true);
}

#[tokio::test]
async fn order_book_api() {
    let kraken = Kraken::new(create_credentials(), create_urls());

    let response = kraken.order_book(&[("pair", "XXRPZUSD")]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();
    assert_eq!(response.len() == 1, true);
    assert_eq!(response.contains_key("XXRPZUSD"), true);

    let order_book = response.get("XXRPZUSD").unwrap();
    assert_eq!(order_book.asks.len() > 0, true);
    assert_eq!(order_book.bids.len() > 0, true);

    let response = kraken.order_book(&[("pair", "XXRPZUSD"), ("count", "2")]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();
    assert_eq!(response.len() == 1, true);
    assert_eq!(response.contains_key("XXRPZUSD"), true);

    let order_book = response.get("XXRPZUSD").unwrap();
    assert_eq!(order_book.asks.len() == 2, true);
    assert_eq!(order_book.bids.len() == 2, true);
}

#[tokio::test]
async fn account_balance_api() {
    let kraken = Kraken::new(create_credentials(), create_urls());
    let response = kraken.account_balance(&[]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();
    assert_eq!(response.contains_key("ZUSD"), true);
}

#[tokio::test]
async fn trade_balance_api() {
    let kraken = Kraken::new(create_credentials(), create_urls());
    let response = kraken.trade_balance(&[]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();
    assert_eq!(response.eb.len() > 0, true);
    assert_eq!(response.tb.len() > 0, true);
    assert_eq!(response.m.len() > 0, true);
    assert_eq!(response.n.len() > 0, true);
    assert_eq!(response.c.len() > 0, true);
    assert_eq!(response.v.len() > 0, true);
    assert_eq!(response.e.len() > 0, true);
    assert_eq!(response.mf.len() > 0, true);

    let response = kraken.trade_balance(&[("asset", "ZUSD")]).await;
    assert_eq!(response.is_ok(), true, "Response: {:?}", response);

    let response = response.unwrap();
    assert_eq!(response.eb.len() > 0, true);
    assert_eq!(response.tb.len() > 0, true);
    assert_eq!(response.m.len() > 0, true);
    assert_eq!(response.n.len() > 0, true);
    assert_eq!(response.c.len() > 0, true);
    assert_eq!(response.v.len() > 0, true);
    assert_eq!(response.e.len() > 0, true);
    assert_eq!(response.mf.len() > 0, true);
}
