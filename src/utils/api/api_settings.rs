use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiSettings {
    pub api_endpoint: String,
    pub api_extra_endpoints: Vec<String>,
    pub cdn_endpoint: String,
    pub cdn_extra_endpoints: Vec<String>,
}

pub fn default_api_endpoint() -> String {
    "https://objectstorage.eu-frankfurt-1.oraclecloud.com/n/froxlhdz2fil/b/bucket-20240619-2047/o/"
        .to_string()
}

pub fn default_api_extra_endpoints() -> Vec<String> {
    vec!["https://objectstorage.eu-frankfurt-1.oraclecloud.com/n/froxlhdz2fil/b/bucket-20240619-2047/o/".to_string()]
}

pub fn default_cdn_endpoint() -> String {
    "https://objectstorage.eu-frankfurt-1.oraclecloud.com/n/froxlhdz2fil/b/bucket-20240619-2047/o/"
        .to_string()
}

pub fn default_cdn_extra_endpoints() -> Vec<String> {
    vec!["https://objectstorage.eu-frankfurt-1.oraclecloud.com/n/froxlhdz2fil/b/bucket-20240619-2047/o/".to_string()]
}

impl Default for ApiSettings {
    fn default() -> Self {
        ApiSettings {
            api_endpoint: default_api_endpoint(),
            api_extra_endpoints: default_api_extra_endpoints(),
            cdn_endpoint: default_cdn_endpoint(),
            cdn_extra_endpoints: default_cdn_extra_endpoints(),
        }
    }
}
