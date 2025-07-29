pub mod jwt;
pub mod oxapay;
pub mod redis;
pub mod server;
pub mod tracing;
use dotenv::dotenv;

#[derive(Clone, Default, Debug)]
pub struct ServiceConfig {
    pub server: server::ServerConfig,
    pub jwt: jwt::JWTConfig,
    pub redis: redis::RedisConfig,
    pub oxapay: oxapay::OxaPayConfig,
}

impl ServiceConfig {
    pub fn init_from_env(&mut self) -> Result<(), String> {
        dotenv().ok();
        self.server.init_from_env()?;
        self.jwt.init_from_env()?;
        self.redis.init_from_env()?;
        self.oxapay.init_from_env()?;
        Ok(())
    }
}
