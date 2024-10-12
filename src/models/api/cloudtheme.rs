use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTheme {
    pub uid: i64,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub primary_color_text: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color_primary: String,
    pub background_color_secondary: String,
    pub background_color_tertiary: String,
    pub primary_grey: String,
    pub secondary_grey: String,
    pub font_size: String,
    pub transparency: bool,
    pub transparency_value: f64,
    pub transparency_blur: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudThemesStatus {
    pub enabled: bool,
}
