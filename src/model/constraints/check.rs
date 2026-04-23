#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Check {
    pub name: String,
    pub condition: String,
}

impl Check {
    pub fn new() -> Self {
        Self {
            name: "check".to_string(),
            condition: "TRUE".to_string(),
        }
    }
}

impl Default for Check {
    fn default() -> Self {
        Self::new()
    }
}
