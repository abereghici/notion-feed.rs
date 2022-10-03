use std::env;

pub const NOTION_API_TOKEN: &str = "NOTION_API_TOKEN";
pub const NOTION_SOURCE_DATABASE_ID: &str = "NOTION_SOURCE_DATABASE_ID";
pub const NOTION_FEED_DATABASE_ID: &str = "NOTION_FEED_DATABASE_ID";

#[derive(Debug)]
pub struct Config {
    pub notion_api_token: String,
    pub notion_source_database_id: String,
    pub notion_feed_database_id: String,
}

impl Config {
    pub fn new(
        notion_source_database_id: Option<String>,
        notion_feed_database_id: Option<String>,
    ) -> Result<Config, String> {
        let notion_api_token = get_config_value(None, NOTION_API_TOKEN)?;
        let notion_source_database_id =
            get_config_value(notion_source_database_id, NOTION_SOURCE_DATABASE_ID)?;
        let notion_feed_database_id =
            get_config_value(notion_feed_database_id, NOTION_FEED_DATABASE_ID)?;

        return Ok(Self {
            notion_api_token,
            notion_source_database_id,
            notion_feed_database_id,
        });
    }
}

fn get_config_value(name: Option<String>, env_name: &str) -> Result<String, String> {
    if let Some(name) = name {
        if !name.is_empty() {
            return Ok(name);
        }
        return Err(format!("Invalid config variable: {:?}", name));
    }

    let env_var = env::var(env_name);

    if let Ok(ref env_var) = env_var {
        if !env_var.is_empty() {
            return Ok(env_var.to_string());
        }
    }

    return Err(format!("Invalid config variable: {:?}", env_var));
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::with_vars;

    #[test]
    fn it_creates_config_when_all_env_vars_are_set() {
        with_vars(
            vec![
                (NOTION_API_TOKEN, Some("token")),
                (NOTION_SOURCE_DATABASE_ID, Some("source db")),
                (NOTION_FEED_DATABASE_ID, Some("feed db")),
            ],
            || {
                let config = Config::new(None, None);
                assert!(config.is_ok());
            },
        );
    }

    #[test]
    fn it_creates_config_when_token_and_args_are_set() {
        with_vars(vec![(NOTION_API_TOKEN, Some("token"))], || {
            let config = Config::new(Some("source db".to_string()), Some("feed db".to_string()));
            assert!(config.is_ok());
        });
    }

    #[test]
    fn it_fails_when_token_is_missing() {
        with_vars(vec![(NOTION_API_TOKEN, Some(""))], || {
            let config = Config::new(Some("source db".to_string()), Some("feed db".to_string()));
            assert!(config.is_err());
        });
    }

    #[test]
    fn it_fails_when_env_vars_and_args_are_missing() {
        with_vars(vec![(NOTION_API_TOKEN, Some(""))], || {
            let config = Config::new(None, Some("".to_string()));
            assert!(config.is_err());
        });
    }

    #[test]
    fn it_creates_config_when_env_vars_are_set_and_args_are_provided() {
        with_vars(
            vec![
                (NOTION_API_TOKEN, Some("token")),
                (NOTION_SOURCE_DATABASE_ID, Some("source db")),
                (NOTION_FEED_DATABASE_ID, Some("feed db")),
            ],
            || {
                let config = Config::new(
                    Some("arg source db".to_string()),
                    Some("arg feed db".to_string()),
                );
                assert!(config.is_ok());

                if let Ok(config) = config {
                    assert_eq!(config.notion_source_database_id, "arg source db");
                    assert_eq!(config.notion_feed_database_id, "arg feed db");
                }
            },
        );
    }
}
