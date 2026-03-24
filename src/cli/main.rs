use clap::{Parser, Subcommand};
use git_reticulator::lattice::affine;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::init();

    match &cli.command {
        Commands::Build { repo, db } => {
            println!("🚀 Starting reticulation process...");
            affine::build_lattice(repo, db);
            println!("✅ Semantic lattice built and stored.");
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
