use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub application_id: u64,
}
