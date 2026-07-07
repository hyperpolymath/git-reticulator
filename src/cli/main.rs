// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
use clap::{Parser, Subcommand, ValueEnum};
use git_reticulator::lattice::SemanticLevel;
use git_reticulator::store::file::FileStore;
use git_reticulator::store::LatticeStore;
use std::path::PathBuf;

/// Default lattice file location relative to the ingested repo.
const DEFAULT_LATTICE_REL: &str = ".git-reticulator/lattice.json";

#[derive(Parser)]
#[command(name = "reticulate")]
#[command(about = "Git-Reticulator: Build and query semantic lattices from git repos", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LevelArg {
    Module,
    File,
    Definition,
    Block,
}

impl From<LevelArg> for SemanticLevel {
    fn from(l: LevelArg) -> Self {
        match l {
            LevelArg::Module => SemanticLevel::Module,
            LevelArg::File => SemanticLevel::File,
            LevelArg::Definition => SemanticLevel::Definition,
            LevelArg::Block => SemanticLevel::Block,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FormatArg {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a semantic lattice from a repo and persist it to a local lattice
    /// file (and optionally to VeriSimDB with --features verisim)
    Build {
        /// Path to the repository to ingest
        #[arg(short, long, default_value = ".")]
        repo: String,
        /// Output lattice file (default: <repo>/.git-reticulator/lattice.json)
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// VeriSimDB base URL (http/https; requires --features verisim)
        #[arg(short, long)]
        db: Option<String>,
    },
    /// Query a built lattice for a token-budgeted context pack
    Query {
        /// Keyword to resolve (case-insensitive substring over node names)
        #[arg(short, long)]
        zoom: String,
        /// Lattice file to query (default: <repo>/.git-reticulator/lattice.json)
        #[arg(short, long)]
        lattice: Option<PathBuf>,
        /// Repository the lattice was built from (locates the default lattice file)
        #[arg(short, long, default_value = ".")]
        repo: String,
        /// Level-of-detail to zoom matches to
        #[arg(long, value_enum, default_value_t = LevelArg::Definition)]
        level: LevelArg,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        /// Token budget for the rendered context pack (chars/4 estimate)
        #[arg(short, long, default_value_t = 2000)]
        budget_tokens: usize,
    },
    /// Start the REST API server for LLM integration
    Api {
        /// PostgreSQL database URI
        #[arg(short, long)]
        db: String,
    },
}

/// Ingest a repository into a lattice. With `--features git-integration` this is
/// git-aware (HEAD tree + commit-history coupling), falling back to a filesystem
/// walk when `repo` is not a git repository; otherwise it is always a walk.
#[cfg(feature = "git-integration")]
fn reticulate_ingest(repo: &str) -> git_reticulator::lattice::Lattice {
    match git_reticulator::ingest::from_git(repo) {
        Ok(lattice) => lattice,
        Err(_) => git_reticulator::ingest::from_path(repo),
    }
}

#[cfg(not(feature = "git-integration"))]
fn reticulate_ingest(repo: &str) -> git_reticulator::lattice::Lattice {
    git_reticulator::ingest::from_path(repo)
}

fn default_lattice_path(repo: &str) -> PathBuf {
    PathBuf::from(repo).join(DEFAULT_LATTICE_REL)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::init();

    match &cli.command {
        Commands::Build { repo, out, db } => {
            println!("🚀 Reticulating {repo} ...");
            let lattice = reticulate_ingest(repo);
            let cond = lattice.condense();
            println!(
                "   {} nodes · {} edges · {} components · acyclic={}",
                lattice.len(),
                lattice.edges().len(),
                cond.num_components,
                cond.is_acyclic()
            );

            let out_path = out.clone().unwrap_or_else(|| default_lattice_path(repo));
            let mut store = FileStore::new(&out_path);
            match store.persist(&lattice) {
                Ok(n) => println!("📦 persisted {n} nodes to {}", out_path.display()),
                Err(e) => {
                    eprintln!("❌ cannot write {}: {e}", out_path.display());
                    std::process::exit(1);
                }
            }

            #[cfg(feature = "verisim")]
            if let Some(db) = db {
                if db.starts_with("http://") || db.starts_with("https://") {
                    let store = git_reticulator::store::verisim::VerisimStore::new(db.clone());
                    match store.persist(&lattice).await {
                        Ok(n) => println!("📦 persisted {n} octads to VeriSimDB ({db})"),
                        Err(e) => eprintln!("⚠️  verisim persist failed: {e}"),
                    }
                }
            }
            #[cfg(not(feature = "verisim"))]
            if let Some(db) = db {
                eprintln!("⚠️  --db {db} ignored: rebuild with --features verisim");
            }

            println!("✅ done.");
        }
        Commands::Query {
            zoom,
            lattice,
            repo,
            level,
            format,
            budget_tokens,
        } => {
            let path = lattice
                .clone()
                .unwrap_or_else(|| default_lattice_path(repo));
            let lat = match FileStore::load(&path) {
                Ok(lat) => lat,
                Err(e) => {
                    eprintln!(
                        "❌ {e}\n   no usable lattice at {} — run `reticulate build --repo {repo}` first",
                        path.display()
                    );
                    std::process::exit(1);
                }
            };
            let result =
                git_reticulator::query::context_pack(&lat, zoom, (*level).into(), *budget_tokens);
            match format {
                FormatArg::Text => print!("{}", git_reticulator::query::render_text(&result)),
                FormatArg::Json => match serde_json::to_string_pretty(&result) {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        eprintln!("❌ cannot serialize result: {e}");
                        std::process::exit(1);
                    }
                },
            }
        }
        Commands::Api { db } => {
            println!("🌐 Starting Git-Reticulator API on http://localhost:8080");
            println!("Using database: {}", db);
            git_reticulator::api::app::start_server(db.clone())
                .await
                .expect("Failed to start API server");
        }
    }
}
