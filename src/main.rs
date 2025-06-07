use std::env;
use std::io::{self, Write};

use emojis_rs::*;
use reqwest;
use serde::{Deserialize, Serialize};
use termimad::*;
use tokio;

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    model: String,
    role: String,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

async fn send_to_claude(
    client: &reqwest::Client,
    api_key: &str,
    user_message: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let request = ClaudeRequest {
        model: "claude-3-haiku-20240307".to_string(),
        max_tokens: 500,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_message.to_string(),
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let claude_response: ClaudeResponse = response.json().await?;
        if let Some(content) = claude_response.content.first() {
            Ok(content.text.clone())
        } else {
            Ok("No response content".to_string())
        }
    } else {
        let error_text = response.text().await?;
        Err(format!("API Error: {}", error_text).into())
    }
}

fn setup_markdown_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    skin.set_headers_fg(rgb(100, 200, 255)); // Light blue headers
    skin.bold.set_fg(rgb(255, 255, 100)); // Yellow bold text
    skin.italic.set_fg(rgb(200, 200, 200)); // Light gray italic
    skin.code_block.set_bg(rgb(40, 40, 40)); // Dark background for code blocks
    skin.code_block.set_fg(rgb(200, 255, 200)); // Light green code text
    skin.inline_code.set_bg(rgb(60, 60, 60)); // Slightly lighter for inline code
    skin.inline_code.set_fg(rgb(255, 200, 100)); // Orange inline code

    skin
}

fn print_styled_response(response: &str) {
    let skin = setup_markdown_skin();

    println!("\n{}", "-".repeat(60));
    skin.print_text(response);
    println!("{}", "-".repeat(60));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    println!("{EMOJI_ROBOT} Welcome to Cogito Lab - Your Rust Claude Chat!");
    println!("{EMOJI_LAMP} Type 'quit' or 'exit' to end the conversation");

    // Get API key
    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY must be set in .env file");

    // Create HTTP client
    let client = reqwest::Client::new();

    // Main chat loop
    loop {
        // Get user input
        print!("\n{EMOJI_BRAIN} You: ");
        io::stdout().flush()?; // Make sure prompt shows immediately

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let user_message = input.trim();

        // Check for quit commands
        if user_message.to_lowercase() == "quit" || user_message.to_lowercase() == "exit" {
            println!("{EMOJI_HAND_WAVE} Goodbye! Thanks for chatting with Claude via Rust!");
            break;
        }

        // Skip empty messages
        if user_message.is_empty() {
            continue;
        }

        // Show thinking indicator
        print!("{EMOJI_THINKING} Claude is thinking...");
        io::stdout().flush()?;

        // Send to Claude and get response
        match send_to_claude(&client, &api_key, user_message).await {
            Ok(response) => {
                print!("\r"); // Clear the "thinking" line
                print_styled_response(&response);
            }
            Err(e) => {
                print!("\r"); // Clear the "thinking" line
                println!("{EMOJI_CROSS} Error: {}", e);
            }
        }
    }

    Ok(())
}

fn mask_key(api_key: &String) -> String {
    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..7], &api_key[api_key.len() - 4..])
    } else {
        "too_short".to_string()
    };

    masked_key
}
