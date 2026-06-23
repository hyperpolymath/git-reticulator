use clap::{Parser, Subcommand};
use git_reticulator::lattice::affine;
use git_reticulator::store::LatticeStore;

#[derive(Parser)]
#[command(name = "reticulate")]
#[command(about = "Git-Reticulator: Build and query semantic lattices from git repos", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a semantic lattice from a git repo
    Build {
        /// Path to the git repository
        #[arg(short, long)]
        repo: String,
        /// PostgreSQL database URI
        #[arg(short, long)]
        db: String,
    },
    /// Query the lattice with a zoom level to minimize token cost
    Query {
        /// Semantic node or keyword to zoom into
        #[arg(short, long)]
        zoom: String,
        /// PostgreSQL database URI
        #[arg(short, long)]
        db: String,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::init();

    match &cli.command {
        Commands::Build { repo, db } => {
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

            #[cfg(feature = "verisim")]
            let to_verisim = if db.starts_with("http://") || db.starts_with("https://") {
                let store = git_reticulator::store::verisim::VerisimStore::new(db.clone());
                match store.persist(&lattice).await {
                    Ok(n) => println!("📦 persisted {n} octads to VeriSimDB ({db})"),
                    Err(e) => eprintln!("⚠️  verisim persist failed: {e}"),
                }
                true
            } else {
                false
            };
            #[cfg(not(feature = "verisim"))]
            let to_verisim = false;

            if !to_verisim {
                let mut store = git_reticulator::store::InMemoryStore::new();
                let n = match store.persist(&lattice) {
                    Ok(n) => n,
                    // InMemoryStore is Infallible — this arm is unreachable.
                    Err(never) => match never {},
                };
                println!("📦 persisted {n} nodes to the in-memory store (target: {db})");
            }
            println!("✅ done.");
        }
        Commands::Query { zoom, db } => {
            println!("🔍 Querying lattice for context: {}", zoom);
            affine::query_lattice(zoom, db);
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
