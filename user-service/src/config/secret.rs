use std::env;
#[derive(Clone, Debug, Default)]
pub struct SecretConfig {
    pub inter_secret_key: String,
}
impl SecretConfig {
    pub fn init_from_env(&mut self) -> Result<(), String> {
        self.inter_secret_key = env::var("INTERNAL_SECRET_KEY")
            .map_err(|_| "INTERNAL_SECRET_KEY not set in environment".to_string())?;

        Ok(())
    }
}
