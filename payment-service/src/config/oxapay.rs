use std::env;
#[derive(Clone, Debug, Default)]
pub struct OxaPayConfig {
    pub merchant_key: String,
    pub return_url: String,
    pub callback_urlbase: String,
}
impl OxaPayConfig {
    pub fn init_from_env(&mut self) -> Result<(), String> {
        self.merchant_key = env::var("MERCHANT_APIKEY")
            .map_err(|_| "MERCHANT_APIKEY not set in environment".to_string())?;

        self.return_url =
            env::var("RETURN_URL").map_err(|_| "RETURN_URL not set in environment".to_string())?;

        self.callback_urlbase = env::var("CALLBACK_URLBASE")
            .map_err(|_| "CALLBACK_URLBASE not set in environment".to_string())?;

        Ok(())
    }
}
