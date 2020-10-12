extern crate kraken_api;

use kraken_api::Kraken;
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
