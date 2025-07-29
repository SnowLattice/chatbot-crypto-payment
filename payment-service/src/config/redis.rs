use std::env;
#[derive(Debug, Clone, Default)]
pub struct RedisConfig {
    pub port: u16,
    pub host: String,
    pub database: String,
}

impl RedisConfig {
    pub fn get_url(&self) -> String {
        Self::create_url(&self.host, self.port, &self.database)
    }

    pub fn create_url(host: &str, port: u16, database_name: &str) -> String {
        format!("redis://{host}:{port}/{database_name}")
    }

    pub fn init_from_env(&mut self) -> Result<(), String> {
        self.host =
            env::var("REDIS_HOST").map_err(|_| "REDIS_HOST not set in environment".to_string())?;

        self.port = env::var("REDIS_PORT")
            .map_err(|_| "REDIS_PORT not set in environment".to_string())?
            .parse::<u16>()
            .map_err(|_| "REDIS_PORT is not a valid u16".to_string())?;

        self.database = env::var("REDIS_DATABASE")
            .map_err(|_| "REDIS_DATABASE not set in environment".to_string())?;

        Ok(())
    }
}
