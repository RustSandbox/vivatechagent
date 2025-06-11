// vivatech planner api

use axum::{routing::post, Json, Router};
use rig::prelude::*;
use rig::{
    agent::Agent,
    completion::Prompt,
    providers::openai,
};
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::SecretStore;
use tracing::info;

mod models;
mod tools;

use models::GeneratePlanRequest;
use tools::QueryVivatechAPI;

// main api endpoint
async fn generate_plan_handler(Json(payload): Json<GeneratePlanRequest>) -> String {
    info!(
        "Received planning request for objective: {}",
        payload.objective
    );

    let openai_client = match initialize_openai_client() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to initialize OpenAI client: {}", e);
            return format!("Error: Failed to initialize AI service - {}", e);
        }
    };

    // simple test mode without tools
    if payload.objective.contains("test simple") {
        info!("Running simple agent test without tools");
        let simple_agent = openai_client
            .agent(openai::GPT_4O)
            .preamble("You are a helpful assistant.")
            .build();

        match simple_agent.prompt(&payload.objective).await {
            Ok(response) => {
                info!("Simple agent response successful");
                return response;
            }
            Err(e) => {
                tracing::error!("Simple agent failed: {}", e);
                return format!("Error: Simple agent failed - {}", e);
            }
        }
    }

    let planner_agent = build_planning_agent(openai_client);
    info!("Planning agent initialized successfully");

    let action_plan = execute_planning_task(&planner_agent, &payload.objective).await;

    info!(
        "Planning task completed, response length: {} chars",
        action_plan.len()
    );
    action_plan
}

// setup openai client from env
fn initialize_openai_client() -> Result<openai::Client, String> {
    match std::env::var("OPENAI_API_KEY") {
        Ok(_) => {
            info!("OpenAI API key found in environment");
            Ok(openai::Client::from_env())
        }
        Err(_) => Err("OPENAI_API_KEY not found in environment".to_string()),
    }
}

// build agent with vivatech context
fn build_planning_agent(client: openai::Client) -> Agent<openai::CompletionModel> {
    const AGENT_INSTRUCTIONS: &str = "\
        You are a helpful assistant for Vivatech 2025 conference planning. \
        Current date: June 11, 2025.\n\n\
        When asked about sessions or events:\n\
        1. Use the query_vivatech_api tool to search for relevant information\n\
        2. Format the results in a clear, organized way for the user\n\
        3. If sessions have dates, note which ones are happening soon";

    client
        .agent(openai::GPT_4O)
        .preamble(AGENT_INSTRUCTIONS)
        .max_tokens(2048)
        .temperature(0.7)
        .tool(QueryVivatechAPI)
        .build()
}

// run the agent with user's request
async fn execute_planning_task(agent: &Agent<openai::CompletionModel>, objective: &str) -> String {
    info!("Executing planning task for: {}", objective);

    match agent.prompt(objective).await {
        Ok(response) => {
            info!("Agent successfully generated response");
            response
        }
        Err(e) => {
            tracing::error!("Agent execution failed: {}", e);
            format!("Error: Failed to generate plan - {}", e)
        }
    }
}

// shuttle entry point
#[shuttle_runtime::main]
async fn axum(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleAxum {
    info!("Starting Vivatech Strategic Planner API v1.0");

    configure_api_keys(&secret_store);

    if let Err(e) = validate_required_configuration() {
        tracing::error!("Configuration validation failed: {}", e);
        panic!("Cannot start service without required configuration: {}", e);
    }
    info!("All required configuration validated");

    let router = build_router();
    Ok(router.into())
}

// load secrets into env vars
fn configure_api_keys(secret_store: &SecretStore) {
    if let Some(api_key) = secret_store.get("OPENAI_API_KEY") {
        std::env::set_var("OPENAI_API_KEY", api_key);
        info!("OpenAI API key configured from secrets");
    } else {
        tracing::warn!("OPENAI_API_KEY not found in secrets - API calls will fail");
    }

    if let Some(api_url) = secret_store.get("VIVATECH_API_URL") {
        std::env::set_var("VIVATECH_API_URL", api_url);
        info!("Vivatech API URL configured from secrets");
    } else {
        tracing::warn!("VIVATECH_API_URL not found in secrets - API calls will fail");
    }

    if let Some(timeout) = secret_store.get("API_TIMEOUT_SECONDS") {
        std::env::set_var("API_TIMEOUT_SECONDS", timeout);
        info!("API timeout configured from secrets");
    }

    if let Some(date) = secret_store.get("CONFERENCE_DATE") {
        std::env::set_var("CONFERENCE_DATE", date);
        info!("Conference date configured from secrets");
    }
}

// setup http routes
fn build_router() -> Router {
    Router::new().route("/generate-plan", post(generate_plan_handler))
}

// check required env vars at startup
fn validate_required_configuration() -> Result<(), String> {
    if std::env::var("OPENAI_API_KEY").is_err() {
        return Err("Missing required configuration: OPENAI_API_KEY. \
             Please set it in Secrets.toml"
            .to_string());
    }

    if std::env::var("VIVATECH_API_URL").is_err() {
        return Err("Missing required configuration: VIVATECH_API_URL. \
             Please set it in Secrets.toml"
            .to_string());
    }

    Ok(())
}
