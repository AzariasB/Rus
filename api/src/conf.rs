pub enum RusEnv {
    RedisUrl,
    DatabaseUrl,
    WebPort,
    WebHost,
}

impl RusEnv {
    fn name(&self) -> &str {
        match *self {
            RusEnv::RedisUrl => "RUS_REDIS_URL",
            RusEnv::DatabaseUrl => "RUS_DATABASE_URL",
            RusEnv::WebPort => "RUS_PORT",
            RusEnv::WebHost => "RUS_HOST",
        }
    }

    pub fn get(&self) -> Option<String> {
        std::env::var(self.name()).ok()
    }

    pub fn get_or(&self, default: String) -> String {
        let var_name = self.name();
        std::env::var(var_name).unwrap_or_else(|err| {
            println!(
                "Failed to get env variable {} (err), using default ({}) instead",
                var_name, err
            );
            default
        })
    }
}
