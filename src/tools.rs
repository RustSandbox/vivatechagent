// agent tools for vivatech api integration

use crate::models::{
    get_current_conference_date, ActionUrgency, VivatechQueryResponse, VivatechSource,
};
use anyhow::Result;
use chrono::NaiveDate;
use regex::Regex;
use reqwest::Client;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

// get api url from env
fn get_vivatech_api_url() -> Result<String, VivatechApiError> {
    std::env::var("VIVATECH_API_URL")
        .map_err(|_| VivatechApiError("VIVATECH_API_URL not found in environment".to_string()))
}

// api timeout with fallback
fn get_api_timeout_seconds() -> u64 {
    std::env::var("API_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30)
}

// tool 1: search vivatech database
#[derive(Debug, Deserialize)]
pub struct QueryVivatechArgs {
    pub query: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Vivatech API Error: {0}")]
pub struct VivatechApiError(String);

#[derive(Serialize, Deserialize)]
pub struct QueryVivatechAPI;

impl Tool for QueryVivatechAPI {
    const NAME: &'static str = "query_vivatech_api";
    type Error = VivatechApiError;
    type Args = QueryVivatechArgs;
    type Output = Vec<VivatechSource>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Searches the Vivatech conference database for sessions and partners related to a query.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search term to find relevant Vivatech sessions or partners"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = create_http_client()?;
        let request_body = json!({ "query": args.query });
        let api_url = get_vivatech_api_url()?;
        let response = make_api_request(&client, &api_url, &request_body).await?;
        let api_response = parse_api_response::<VivatechQueryResponse>(response).await?;
        Ok(api_response.sources)
    }
}

// tool 2: assess event timeliness
#[derive(Debug, Deserialize)]
pub struct AssessTimelinessArgs {
    pub events: Vec<VivatechSource>,
}

#[derive(Debug, Serialize)]
pub struct TimelinessResult {
    pub source_id: String,
    pub urgency: ActionUrgency,
    pub description: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse date from event text")]
pub struct DateParseError;

#[derive(Serialize, Deserialize)]
pub struct AssessTimeliness;

impl Tool for AssessTimeliness {
    const NAME: &'static str = "assess_event_timeliness";
    type Error = DateParseError;
    type Args = AssessTimelinessArgs;
    type Output = Vec<TimelinessResult>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Analyzes a list of Vivatech events to determine their urgency based on the current date (June 11, 2025). Use this to prioritize actions.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "events": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { 
                                    "type": "string",
                                    "description": "Unique identifier of the event"
                                },
                                "text_chunk": { 
                                    "type": "string",
                                    "description": "Text content describing the event"
                                },
                                "source_table": {
                                    "type": "string",
                                    "description": "Type of source (e.g., sessions, partners)"
                                },
                                "score": {
                                    "type": "number",
                                    "description": "Relevance score"
                                }
                            },
                            "required": ["id", "text_chunk"]
                        },
                        "description": "List of events to assess for timeliness"
                    }
                },
                "required": ["events"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_date = get_current_conference_date();
        let mut results = Vec::new();

        for event in args.events {
            let (urgency, description) = analyze_event_urgency(&event.text_chunk, current_date);
            results.push(TimelinessResult {
                source_id: event.id,
                urgency,
                description,
            });
        }

        Ok(results)
    }
}

// helper functions
fn create_http_client() -> Result<Client, VivatechApiError> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(get_api_timeout_seconds()))
        .build()
        .map_err(|e| VivatechApiError(format!("Failed to create HTTP client: {}", e)))
}

async fn make_api_request(
    client: &Client,
    url: &str,
    body: &serde_json::Value,
) -> Result<reqwest::Response, VivatechApiError> {
    let response = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| VivatechApiError(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(VivatechApiError(format!(
            "API returned error status: {}",
            response.status()
        )));
    }

    Ok(response)
}

async fn parse_api_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
) -> Result<T, VivatechApiError> {
    response
        .json()
        .await
        .map_err(|e| VivatechApiError(format!("Failed to parse JSON response: {}", e)))
}

// check event urgency based on date
fn analyze_event_urgency(text: &str, current_date: NaiveDate) -> (ActionUrgency, String) {
    match extract_date_from_text(text) {
        Some(event_date) => {
            let days_until_event = (event_date - current_date).num_days();
            match days_until_event {
                0 => (
                    ActionUrgency::Immediate,
                    "This event is happening TODAY - immediate action required!".to_string(),
                ),
                1 => (
                    ActionUrgency::Soon,
                    "This event is happening TOMORROW - plan accordingly.".to_string(),
                ),
                d if d > 0 => (
                    ActionUrgency::Normal,
                    format!("This event is in {} days - normal priority.", d),
                ),
                _ => (
                    ActionUrgency::Normal,
                    "This event has already passed.".to_string(),
                ),
            }
        }
        None => (
            ActionUrgency::Normal,
            "No specific date found - treating as normal priority.".to_string(),
        ),
    }
}

// extract dates from text
fn extract_date_from_text(text: &str) -> Option<NaiveDate> {
    // try "June 12" format
    let month_day_pattern = r"(January|February|March|April|May|June|July|August|September|October|November|December)\s+(\d{1,2})";
    if let Ok(regex) = Regex::new(month_day_pattern) {
        if let Some(captures) = regex.captures(text) {
            if let Some(date) = extract_month_day_date(&captures) {
                return Some(date);
            }
        }
    }

    // try "12th June" format
    let day_month_pattern = r"(\d{1,2})(?:st|nd|rd|th)?\s+(January|February|March|April|May|June|July|August|September|October|November|December)";
    if let Ok(regex) = Regex::new(day_month_pattern) {
        if let Some(captures) = regex.captures(text) {
            if let Some(date) = extract_day_month_date(&captures) {
                return Some(date);
            }
        }
    }

    None
}

fn extract_month_day_date(captures: &regex::Captures) -> Option<NaiveDate> {
    let month_str = captures.get(1)?.as_str();
    let day_str = captures.get(2)?.as_str();

    let month_num = month_name_to_number(month_str)?;
    let day = day_str.parse::<u32>().ok()?;

    NaiveDate::from_ymd_opt(2025, month_num, day)
}

fn extract_day_month_date(captures: &regex::Captures) -> Option<NaiveDate> {
    let day_str = captures.get(1)?.as_str();
    let month_str = captures.get(2)?.as_str();

    let day = day_str.parse::<u32>().ok()?;
    let month_num = month_name_to_number(month_str)?;

    NaiveDate::from_ymd_opt(2025, month_num, day)
}

// convert month names to numbers
fn month_name_to_number(month: &str) -> Option<u32> {
    match month.to_lowercase().as_str() {
        "january" => Some(1),
        "february" => Some(2),
        "march" => Some(3),
        "april" => Some(4),
        "may" => Some(5),
        "june" => Some(6),
        "july" => Some(7),
        "august" => Some(8),
        "september" => Some(9),
        "october" => Some(10),
        "november" => Some(11),
        "december" => Some(12),
        _ => None,
    }
}
