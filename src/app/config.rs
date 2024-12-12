use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub rabbit_mq: RabbitMQSettings,
}

impl Settings {
    pub fn load_configuration() -> Settings {
        let config_file = config_file();

        let config: Result<Settings, _> = config::Config::builder()
            .add_source(config::File::new(&config_file, config::FileFormat::Yaml))
            .build()
            .and_then(|b| b.try_deserialize());
        if config.is_ok() {
            config.unwrap()
        } else {
            Settings::default()
        }
    }

    pub fn write_configuration(&self) -> std::io::Result<()> {
        let config_s = serde_yaml::to_string(self).expect("Failed to serialize config");
        std::fs::write(config_file(), config_s)
    }
}

fn config_file() -> String {
    let local_config_dir = dirs::config_local_dir().expect("Failed to get config dir!");
    let config_file = local_config_dir
        .join("bucklog/config.yaml")
        .to_str()
        .expect("Failed to get config path")
        .to_string();
    config_file
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RabbitMQSettings {
    pub host: String,
    pub vhost: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl RabbitMQSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
        .into()
    }
}

mod arc_rwlock_serde {
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, RwLock};

    pub fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*val.read().unwrap(), s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }
}
