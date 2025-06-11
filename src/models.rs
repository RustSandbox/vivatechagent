// data models for vivatech api

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// vivatech 2025 defaults
const VIVATECH_YEAR: i32 = 2025;
const CURRENT_MONTH: u32 = 6; // June
const CURRENT_DAY: u32 = 11;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActionUrgency {
    Immediate,
    Soon,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivatechSource {
    pub id: String,
    #[serde(default)]
    pub source_table: String,
    #[serde(default)]
    pub score: f32,
    pub text_chunk: String,
}

#[derive(Debug, Deserialize)]
pub struct VivatechMetadata {
    pub search_mode: String,
    pub sources_found: u32,
}

#[derive(Debug, Deserialize)]
pub struct VivatechQueryResponse {
    #[allow(dead_code)]
    pub answer: String,
    pub sources: Vec<VivatechSource>,
    #[allow(dead_code)]
    pub metadata: VivatechMetadata,
}

#[derive(Debug, Deserialize)]
pub struct GeneratePlanRequest {
    pub objective: String,
}

// get conference date from env or use default
pub fn get_current_conference_date() -> NaiveDate {
    if let Ok(date_str) = std::env::var("CONFERENCE_DATE") {
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            return date;
        }
    }
    
    NaiveDate::from_ymd_opt(VIVATECH_YEAR, CURRENT_MONTH, CURRENT_DAY)
        .expect("June 11, 2025 is a valid date")
}
