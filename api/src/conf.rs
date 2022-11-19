use log::warn;

pub enum RusConf {
    RedisUrl,
    DatabaseUrl,
    WebPort,
    WebHost,
    LinkDaysLifeTime,
}

impl RusConf {
    fn name(&self) -> &str {
        match *self {
            RusConf::RedisUrl => "RUS_REDIS_URL",
            RusConf::DatabaseUrl => "RUS_DATABASE_URL",
            RusConf::WebPort => "RUS_PORT",
            RusConf::WebHost => "RUS_HOST",
            RusConf::LinkDaysLifeTime => "RUS_LINKS_LIFETIME",
        }
    }

    pub fn get(&self) -> Option<String> {
        std::env::var(self.name()).ok()
    }

    pub fn get_i64_or(&self, default: i64) -> i64 {
        self.get()
            .and_then(|val| {
                val.parse::<i64>()
                    .map_err(|err| {
                        warn!("{} is not a valid int ({})", val, err);
                    })
                    .ok()
            })
            .unwrap_or(default)
    }

    pub fn get_or(&self, default: String) -> String {
        let var_name = self.name();
        std::env::var(var_name).unwrap_or_else(|err| {
            warn!(
                "Failed to get env variable {} (err), using default ({}) instead",
                var_name, err
            );
            default
        })
    }
}
