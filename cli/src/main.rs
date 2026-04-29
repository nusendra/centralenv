mod config;
mod client;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "centralenv", about = "Centralized .env manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save server URL and token to config
    Login {
        /// Server base URL (e.g. http://100.x.x.x:3001)
        #[arg(long)]
        url: String,
        /// API token
        #[arg(long)]
        token: String,
    },
    /// Pull env vars and write to a .env file
    Pull {
        /// Project slug
        project: String,
        /// Environment name (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Output file (default: .env)
        #[arg(short, long, default_value = ".env")]
        output: PathBuf,
    },
    /// Inject env vars into a subprocess without writing to disk
    Run {
        /// Project slug
        project: String,
        /// Environment name (default: development)
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Command to run
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { url, token } => {
            let cfg = config::Config { server_url: url, token };
            cfg.save()?;
            println!("Logged in and config saved.");
        }
        Commands::Pull { project, env, output } => {
            let cfg = config::Config::load()?;
            let vars = client::fetch_env(&cfg, &project, &env).await?;
            write_dotenv(&output, &vars)?;
            println!("Wrote {} variables to {}", vars.len(), output.display());
        }
        Commands::Run { project, env, cmd } => {
            let cfg = config::Config::load()?;
            let vars = client::fetch_env(&cfg, &project, &env).await?;

            let mut command = std::process::Command::new(&cmd[0]);
            command.args(&cmd[1..]);
            for (k, v) in &vars {
                command.env(k, v);
            }
            let status = command.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}

fn write_dotenv(path: &PathBuf, vars: &HashMap<String, String>) -> Result<()> {
    let mut lines: Vec<String> = vars
        .iter()
        .map(|(k, v)| {
            // Quote values that contain spaces or special chars
            if v.contains('"') || v.contains('\\') {
                format!("{}={}", k, v)
            } else if v.contains(' ') || v.contains('#') || v.contains('=') {
                format!("{}=\"{}\"", k, v)
            } else {
                format!("{}={}", k, v)
            }
        })
        .collect();
    lines.sort();
    std::fs::write(path, lines.join("\n") + "\n")?;
    Ok(())
}
