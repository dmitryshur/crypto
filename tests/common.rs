extern crate kraken_api;

use kraken_api::Credentials;

pub fn create_credentials() -> Credentials {
    Credentials::new(
        "YHtLCbf8IEbiLyiLE6iD//i99jsL4mi1+9Nh9vWBsp+KAx/GpdWi/+Yt".to_string(),
        "fB9sLSGJrPjAwtxrId/mhEmg7iZSPzyNYvnVbMYOGWreS6k17JuFoKF94xG2BB25rsM1hy5v6Eja5S+A4E7ckA=="
            .to_string(),
    )
}
