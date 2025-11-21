use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "ac-pcap-parser")]
#[command(about = "Parse Asheron's Call PCAP files", long_about = None)]
pub struct Cli {
    /// PCAP file to parse
    #[arg(short, long, default_value = "pkt_2025-11-18_1763490291_log.pcap")]
    pub file: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show messages in JSON format
    Messages {
        /// Filter by message type (substring match)
        #[arg(short = 't', long)]
        filter_type: Option<String>,

        /// Filter by direction (Send/Recv)
        #[arg(short = 'd', long)]
        direction: Option<DirectionFilter>,

        /// Sort by field
        #[arg(short, long, default_value = "id")]
        sort: SortField,

        /// Reverse sort order
        #[arg(short, long)]
        reverse: bool,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Output format
        #[arg(short, long, default_value = "jsonl")]
        output: OutputFormat,
    },

    /// Show fragments/packets in JSON format
    Fragments {
        /// Filter by direction (Send/Recv)
        #[arg(short = 'd', long)]
        direction: Option<DirectionFilter>,

        /// Sort by field
        #[arg(short, long, default_value = "id")]
        sort: FragmentSortField,

        /// Reverse sort order
        #[arg(short, long)]
        reverse: bool,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Output format
        #[arg(short, long, default_value = "jsonl")]
        output: OutputFormat,
    },

    /// Show summary statistics
    Summary,

    /// Interactive TUI mode
    Tui,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum DirectionFilter {
    Send,
    Recv,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum SortField {
    Id,
    Type,
    Direction,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum FragmentSortField {
    Id,
    Sequence,
    Direction,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Jsonl,
    Json,
    Table,
}
