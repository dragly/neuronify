use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ThirdParty {
    pub packages: Vec<Package>,
    pub texts: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub license: String,
    pub texts: Vec<usize>,
}
