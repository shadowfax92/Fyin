use clap::Parser;

/// fyin.app - Open source CLI alternative to Perplexity AI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Search Query
    #[arg(short, long)]
    pub query: String,

    /// Number of search results to parse
    #[arg(short, long, default_value_t = 10)]
    pub search: usize,

    /// Register a new user
    #[arg(long)]
    pub register: Option<String>,

    /// Login as an existing user
    #[arg(long)]
    pub login: Option<String>,
}
