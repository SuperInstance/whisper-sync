use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod whisper;
mod plato_transport;
mod delivery;
mod inbox;
mod filter;

use whisper::{Whisper, WhisperType};
use plato_transport::PlatoTransport;
use delivery::DeliveryMode;

#[derive(Parser)]
#[command(name = "whisper-sync")]
#[command(about = "Murmurer's ambient whisper protocol over PLATO rooms")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a whisper
    Send {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        whisper_type: String,
        #[arg(long)]
        content: String,
        /// TTL in seconds. Defaults to per-type optimal value:
        /// Status=60s, Alert=60s, Trust=5min, Discovery=5min, Help=30min, Insight=4h
        #[arg(long)]
        ttl: Option<u64>,
    },
    /// Listen for whispers addressed to this agent
    Listen {
        #[arg(long)]
        agent_id: String,
        #[arg(long, default_value = "10")]
        interval: u64,
    },
}

fn parse_whisper_type(type_str: &str, content: &str) -> Result<WhisperType, String> {
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("invalid JSON content: {}", e))?;

    match type_str {
        "status" => Ok(WhisperType::Status {
            health: json.get("health").and_then(|v| v.as_f64()).unwrap_or(1.0),
            load: json.get("load").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }),
        "discovery" => Ok(WhisperType::Discovery {
            service: json.get("service").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            endpoint: json.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }),
        "help" => Ok(WhisperType::Help {
            question: json.get("question").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tags: json.get("tags").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                .unwrap_or_default(),
        }),
        "insight" => Ok(WhisperType::Insight {
            summary: json.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            confidence: json.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
            source_theorem: json.get("source_theorem").and_then(|v| v.as_str()).map(String::from),
        }),
        "trust" => Ok(WhisperType::Trust {
            agent: json.get("agent").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            trust_score: json.get("trust_score").and_then(|v| v.as_f64()).unwrap_or(0.5),
        }),
        "alert" => Ok(WhisperType::Alert {
            severity: json.get("severity").and_then(|v| v.as_str()).unwrap_or("info").to_string(),
            message: json.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }),
        _ => Err(format!("unknown whisper type: {}", type_str)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("whisper_sync=info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send { from, to, whisper_type, content, ttl } => {
            let transport = PlatoTransport::new();
            let wt = parse_whisper_type(&whisper_type, &content)?;
            let effective_ttl = ttl.unwrap_or_else(|| wt.default_ttl_seconds());
            let whisper = Whisper::new(from, to, wt, effective_ttl);

            tracing::info!("sending {} whisper", whisper_type);
            transport.submit_whisper(&whisper).await?;
            println!("whisper sent successfully");
        }
        Commands::Listen { agent_id, interval } => {
            let transport = PlatoTransport::new();
            let inbox = inbox::Inbox::new(transport, agent_id.clone());

            tracing::info!("listening for whispers as {}", agent_id);
            inbox.listen(interval, |whispers| async move {
                for w in whispers {
                    println!("[whisper] {}: {:?}", w.from, w.whisper_type);
                }
            }).await;
        }
    }

    Ok(())
}
