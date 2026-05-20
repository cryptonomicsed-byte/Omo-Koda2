use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use omokoda_core::{
    interpreter::Steward,
    parser::{parse, Statement, ThinkModifiers},
};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aether", version, about = "Ọmọ Kọ́dà sovereign agent CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Birth a new sovereign agent
    Birth {
        /// Agent name
        name: String,
        /// Optional metadata as key=value pairs
        #[arg(short, long, value_parser = parse_kv)]
        meta: Vec<(String, String)>,
    },
    /// Send a think primitive to the active agent
    Think {
        /// Prompt text
        prompt: String,
        /// Private mode (local provider only)
        #[arg(short, long)]
        private: bool,
    },
    /// Execute an act primitive via the active agent
    Act {
        /// Tool name
        tool: String,
        /// Tool parameters (JSON or plain string)
        #[arg(default_value = "{}")]
        params: String,
        /// Enable sandbox mode
        #[arg(short, long)]
        sandbox: bool,
    },
    /// Run a .swibe script file
    Run {
        /// Path to .swibe script, or "-" / "--stdin" to read from stdin
        #[arg(default_value = "-")]
        script: String,
        /// Read from stdin (alias for script="-")
        #[arg(long)]
        stdin: bool,
    },
    /// Print agent status
    Status,
    /// Start the interactive REPL
    Repl,
    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
}

#[derive(Subcommand)]
enum SessionAction {
    /// List persisted sessions
    List,
    /// Resume a session by agent-id prefix
    Resume { id: String },
    /// Archive (seal) the current session
    Archive,
}

fn parse_kv(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("expected key=value, got '{s}'"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Birth { name, meta }) => {
            let mut steward = Steward::new();
            run_statement(
                &mut steward,
                Statement::Birth {
                    name,
                    metadata: meta
                        .into_iter()
                        .map(|(k, v)| omokoda_core::parser::MetadataPair { key: k, value: v })
                        .collect(),
                },
            )
            .await?;
        }

        Some(Command::Think { prompt, private }) => {
            let mut steward = load_or_new_steward();
            run_statement(
                &mut steward,
                Statement::Think {
                    prompt,
                    private,
                    modifiers: ThinkModifiers::default(),
                },
            )
            .await?;
        }

        Some(Command::Act {
            tool,
            params,
            sandbox,
        }) => {
            let mut steward = load_or_new_steward();
            run_statement(
                &mut steward,
                Statement::Act {
                    tool,
                    params,
                    sandbox,
                },
            )
            .await?;
        }

        Some(Command::Run { script, stdin }) => {
            let source = if stdin || script == "-" {
                use std::io::Read;
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                std::fs::read_to_string(&script)
                    .with_context(|| format!("reading script '{script}'"))?
            };
            run_script(source).await?;
        }

        Some(Command::Status) => {
            let mut steward = load_or_new_steward();
            run_slash(&mut steward, "status", None).await?;
        }

        Some(Command::Repl) | None => {
            repl().await?;
        }

        Some(Command::Session { action }) => match action {
            SessionAction::List => session_list()?,
            SessionAction::Resume { id } => session_resume(&id).await?,
            SessionAction::Archive => {
                let mut steward = load_or_new_steward();
                run_slash(&mut steward, "seal", None).await?;
            }
        },
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

async fn repl() -> Result<()> {
    println!(
        "{}",
        "Ọmọ Kọ́dà  •  Àṣẹ CLI  •  type 'help' or Ctrl-D to exit"
            .bold()
            .cyan()
    );
    let mut rl = DefaultEditor::new()?;
    let mut steward = load_or_new_steward();

    loop {
        let prompt = agent_prompt(&steward);
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&line);
                if line == "exit" || line == "quit" {
                    break;
                }
                if let Err(e) = handle_repl_line(&mut steward, &line).await {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn handle_repl_line(steward: &mut Steward, line: &str) -> Result<()> {
    if line.starts_with('/') {
        let mut parts = line[1..].splitn(2, ' ');
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().map(|s| s.to_string());
        run_slash(steward, cmd, arg).await
    } else {
        let stmts = parse(line).map_err(|e| anyhow::anyhow!("{}", e))?;
        for stmt in stmts {
            run_statement(steward, stmt).await?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Script runner
// ---------------------------------------------------------------------------

async fn run_script(source: String) -> Result<()> {
    let mut steward = load_or_new_steward();
    let stmts = parse(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    for stmt in stmts {
        run_statement(&mut steward, stmt).await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Core dispatch
// ---------------------------------------------------------------------------

async fn run_statement(steward: &mut Steward, stmt: Statement) -> Result<()> {
    let result = steward
        .dispatch(stmt)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(output) = result.tool_output {
        if result.private_mode {
            println!("{}", output.dimmed());
        } else {
            println!("{}", output);
        }
    }
    if let Some(receipt) = result.receipt {
        println!("{}  {}", "receipt:".dimmed(), receipt.receipt_id.dimmed());
    }
    Ok(())
}

async fn run_slash(steward: &mut Steward, cmd: &str, arg: Option<String>) -> Result<()> {
    use omokoda_core::parser::Statement;
    let stmt = Statement::SlashCmd {
        command: cmd.to_string(),
        arg,
    };
    run_statement(steward, stmt).await
}

// ---------------------------------------------------------------------------
// Session helpers
// ---------------------------------------------------------------------------

fn session_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".omokoda")
        .join("sessions")
}

fn load_or_new_steward() -> Steward {
    Steward::new()
}

fn session_list() -> Result<()> {
    let dir = session_dir();
    if !dir.exists() {
        println!("No sessions found ({})", dir.display());
        return Ok(());
    }
    let mut count = 0usize;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        println!("  {}", name.to_string_lossy().cyan());
        count += 1;
    }
    if count == 0 {
        println!("No sessions found.");
    }
    Ok(())
}

async fn session_resume(id: &str) -> Result<()> {
    println!("{} {}", "Resuming session".yellow(), id);
    let mut steward = load_or_new_steward();
    run_slash(&mut steward, "status", None).await
}

fn agent_prompt(steward: &Steward) -> String {
    if let Some(agent) = steward.agent_state() {
        format!("{}> ", agent.name().cyan())
    } else {
        "aether> ".to_string()
    }
}
