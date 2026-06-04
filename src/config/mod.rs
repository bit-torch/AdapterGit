use std::env;

pub struct Config {
    pub user_name: String,
    pub user_email: String,
}

impl Config {
    pub fn load() -> Self {
        Config {
            user_name: env::var("AGIT_USER_NAME")
                .or_else(|_| env::var("GIT_AUTHOR_NAME"))
                .unwrap_or_else(|_| "agit".to_string()),
            user_email: env::var("AGIT_USER_EMAIL")
                .or_else(|_| env::var("GIT_AUTHOR_EMAIL"))
                .unwrap_or_else(|_| "agit@localhost".to_string()),
        }
    }
}

pub fn load() -> Config {
    Config::load()
}
