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

impl Package {
    pub fn title(&self) -> String {
        format!("{} {}", self.name, self.version)
    }
}

const COMPRESSED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/third-party-licenses.deflate"));

pub fn load() -> ThirdParty {
    let list = miniz_oxide::inflate::decompress_to_vec(COMPRESSED).unwrap();
    serde_json::from_slice(&list).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_carries_the_generated_list() {
        let generated = include_str!("../assets/third-party-licenses.json");
        assert_eq!(load(), serde_json::from_str(generated).unwrap());
    }

    #[test]
    fn every_package_comes_with_license_texts() {
        let third_party = load();
        for package in &third_party.packages {
            assert!(
                !package.texts.is_empty(),
                "{} has no texts",
                package.title()
            );
            for &text in &package.texts {
                assert!(text < third_party.texts.len());
            }
        }
    }
}
