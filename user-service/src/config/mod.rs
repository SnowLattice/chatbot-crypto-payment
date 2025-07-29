pub mod db;
pub mod jwt;
pub mod redis;
pub mod secret;
pub mod server;
pub mod tracing;
use dotenv::dotenv;
use std::env;

#[derive(Clone, Default, Debug)]
pub struct ServiceConfig {
    pub db: db::DatabaseConfig,
    pub redis: redis::RedisConfig,
    pub server: server::ServerConfig,
    pub secret: secret::SecretConfig,
    pub jwt: jwt::JWTConfig,
    pub bot_token: String,
    pub charged_credit: i64,
}
impl ServiceConfig {
    pub fn init_from_env(&mut self) -> Result<(), String> {
        dotenv().ok();
        self.db.init_from_env()?;
        self.redis.init_from_env()?;
        self.server.init_from_env()?;
        self.jwt.init_from_env()?;
        self.secret.init_from_env()?;
        self.charged_credit = 1000;
        self.bot_token =
            env::var("BOT_TOKEN").map_err(|_| "BOT_TOKEN not set in environment".to_string())?;
        Ok(())
    }
}
