use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use std::fs::File;

use lib::{messages::ParsedMessage, Direction, PacketParser, ParsedPacket};

mod filter;
mod tui;

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

        /// Filter by opcode (hex like 0xF7B1 or decimal like 63409)
        #[arg(short = 'c', long)]
        filter_opcode: Option<String>,

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

    /// Show weenies/objects aggregated from the PCAP
    Weenies {
        /// Filter by ObjectID (exact match or substring)
        #[arg(short = 'i', long)]
        object_id: Option<String>,

        /// Filter by name (substring match)
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// Sort by field
        #[arg(short, long, default_value = "id")]
        sort: WeenieSortField,

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
pub enum WeenieSortField {
    Id,
    Name,
    Properties,
    Messages,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Jsonl,
    Json,
    Table,
}

fn print_summary(
    packets: &[ParsedPacket],
    messages: &[ParsedMessage],
    weenie_db: &lib::weenie::WeenieDatabase,
) {
    println!("=== PCAP Summary ===\n");

    println!("Packets: {}", packets.len());
    println!("Messages: {}", messages.len());
    println!("Weenies: {}", weenie_db.count());

    let send_packets = packets
        .iter()
        .filter(|p| matches!(p.direction, Direction::Send))
        .count();
    let recv_packets = packets
        .iter()
        .filter(|p| matches!(p.direction, Direction::Recv))
        .count();
    println!("\nPackets by Direction:");
    println!("  Send (C→S): {send_packets}");
    println!("  Recv (S→C): {recv_packets}");

    let send_msgs = messages.iter().filter(|m| m.direction == "Send").count();
    let recv_msgs = messages.iter().filter(|m| m.direction == "Recv").count();
    println!("\nMessages by Direction:");
    println!("  Send (C→S): {send_msgs}");
    println!("  Recv (S→C): {recv_msgs}");

    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    for msg in messages {
        *type_counts.entry(&msg.message_type).or_insert(0) += 1;
    }

    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1));

    println!("\nMessage Types (top 20):");
    for (t, count) in sorted_types.iter().take(20) {
        println!("  {t:40} {count:>5}");
    }

    if sorted_types.len() > 20 {
        println!("  ... and {} more types", sorted_types.len() - 20);
    }
}

#[allow(clippy::too_many_arguments)]
fn output_messages(
    messages: &[ParsedMessage],
    filter_type: Option<&str>,
    filter_opcode: Option<&str>,
    direction: Option<DirectionFilter>,
    sort: SortField,
    reverse: bool,
    limit: Option<usize>,
    output: OutputFormat,
) {
    // Parse opcode filter if provided
    let opcode_filter: Option<u32> =
        filter_opcode.and_then(|s| filter::parse_opcode_filter(s).ok());

    let mut filtered: Vec<&ParsedMessage> = messages
        .iter()
        .filter(|m| {
            if let Some(ft) = filter_type {
                if !m.message_type.to_lowercase().contains(&ft.to_lowercase()) {
                    return false;
                }
            }
            if let Some(oc) = opcode_filter {
                if let Some(msg_opcode) = filter::opcode_str_to_u32(&m.opcode) {
                    if msg_opcode != oc {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if let Some(d) = direction {
                match d {
                    DirectionFilter::Send => {
                        if m.direction != "Send" {
                            return false;
                        }
                    }
                    DirectionFilter::Recv => {
                        if m.direction != "Recv" {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort {
            SortField::Id => a.id.cmp(&b.id),
            SortField::Type => a.message_type.cmp(&b.message_type),
            SortField::Direction => a.direction.cmp(&b.direction),
        };
        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    if let Some(lim) = limit {
        filtered.truncate(lim);
    }

    match output {
        OutputFormat::Jsonl => {
            for msg in filtered {
                println!("{}", serde_json::to_string(&msg).unwrap());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
        }
        OutputFormat::Table => {
            println!("{:>6}  {:40}  {:>6}  {:>10}", "ID", "Type", "Dir", "OpCode");
            println!("{}", "-".repeat(70));
            for msg in filtered {
                println!(
                    "{:>6}  {:40}  {:>6}  {:>10}",
                    msg.id,
                    truncate(&msg.message_type, 40),
                    msg.direction,
                    msg.opcode
                );
            }
        }
    }
}

fn output_fragments(
    packets: &[ParsedPacket],
    direction: Option<DirectionFilter>,
    sort: FragmentSortField,
    reverse: bool,
    limit: Option<usize>,
    output: OutputFormat,
) {
    let mut filtered: Vec<&ParsedPacket> = packets
        .iter()
        .filter(|p| {
            if let Some(d) = direction {
                match d {
                    DirectionFilter::Send => {
                        if !matches!(p.direction, Direction::Send) {
                            return false;
                        }
                    }
                    DirectionFilter::Recv => {
                        if !matches!(p.direction, Direction::Recv) {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort {
            FragmentSortField::Id => a.id.cmp(&b.id),
            FragmentSortField::Sequence => a.header.sequence.cmp(&b.header.sequence),
            FragmentSortField::Direction => {
                format!("{:?}", a.direction).cmp(&format!("{:?}", b.direction))
            }
        };
        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    if let Some(lim) = limit {
        filtered.truncate(lim);
    }

    match output {
        OutputFormat::Jsonl => {
            for pkt in filtered {
                println!("{}", serde_json::to_string(&pkt).unwrap());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
        }
        OutputFormat::Table => {
            println!(
                "{:>6}  {:>10}  {:>6}  {:>12}  {:>6}",
                "ID", "Seq", "Dir", "Flags", "Size"
            );
            println!("{}", "-".repeat(50));
            for pkt in filtered {
                println!(
                    "{:>6}  {:>10}  {:>6}  {:>12}  {:>6}",
                    pkt.id,
                    pkt.header.sequence,
                    format!("{:?}", pkt.direction),
                    format!("{:08X}", pkt.header.flags.bits()),
                    pkt.header.size
                );
            }
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[allow(clippy::too_many_arguments)]
fn output_weenies(
    weenie_db: &lib::weenie::WeenieDatabase,
    object_id_filter: Option<&str>,
    name_filter: Option<&str>,
    sort: WeenieSortField,
    reverse: bool,
    limit: Option<usize>,
    output: OutputFormat,
) {
    let mut weenies: Vec<_> = weenie_db.sorted_weenies();

    // Apply filters
    weenies.retain(|w| {
        if let Some(id_filter) = object_id_filter {
            if !w.object_id.to_string().contains(id_filter) {
                return false;
            }
        }
        if let Some(name_filter) = name_filter {
            if let Some(ref name) = w.name {
                if !name.to_lowercase().contains(&name_filter.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    });

    // Sort
    weenies.sort_by(|a, b| {
        let cmp = match sort {
            WeenieSortField::Id => a.object_id.cmp(&b.object_id),
            WeenieSortField::Name => a
                .name
                .as_deref()
                .unwrap_or("")
                .cmp(b.name.as_deref().unwrap_or("")),
            WeenieSortField::Properties => {
                let a_count = a.int_properties.len()
                    + a.int64_properties.len()
                    + a.bool_properties.len()
                    + a.float_properties.len()
                    + a.string_properties.len()
                    + a.data_id_properties.len()
                    + a.instance_id_properties.len();
                let b_count = b.int_properties.len()
                    + b.int64_properties.len()
                    + b.bool_properties.len()
                    + b.float_properties.len()
                    + b.string_properties.len()
                    + b.data_id_properties.len()
                    + b.instance_id_properties.len();
                a_count.cmp(&b_count)
            }
            WeenieSortField::Messages => a.message_count.cmp(&b.message_count),
        };
        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    if let Some(lim) = limit {
        weenies.truncate(lim);
    }

    match output {
        OutputFormat::Jsonl => {
            for weenie in weenies {
                println!("{}", serde_json::to_string(&weenie).unwrap());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&weenies).unwrap());
        }
        OutputFormat::Table => {
            println!(
                "{:>10}  {:40}  {:>6}  {:>6}",
                "ObjectID", "Name", "Props", "Msgs"
            );
            println!("{}", "-".repeat(70));
            for weenie in weenies {
                let prop_count = weenie.int_properties.len()
                    + weenie.int64_properties.len()
                    + weenie.bool_properties.len()
                    + weenie.float_properties.len()
                    + weenie.string_properties.len()
                    + weenie.data_id_properties.len()
                    + weenie.instance_id_properties.len();
                println!(
                    "{:>10}  {:40}  {:>6}  {:>6}",
                    weenie.object_id,
                    truncate(weenie.name.as_deref().unwrap_or("<unknown>"), 40),
                    prop_count,
                    weenie.message_count
                );
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut parser = PacketParser::new();

    eprintln!("Parsing PCAP file: {}", cli.file);

    let file = File::open(&cli.file).context("Failed to open pcap file")?;
    let (packets, messages, weenie_db) = parser
        .parse_pcap(file)
        .context("Failed to parse pcap file")?;

    eprintln!(
        "Found {} packets, {} messages, {} weenies",
        packets.len(),
        messages.len(),
        weenie_db.count()
    );

    match cli.command {
        Some(Commands::Messages {
            filter_type,
            filter_opcode,
            direction,
            sort,
            reverse,
            limit,
            output,
        }) => {
            output_messages(
                &messages,
                filter_type.as_deref(),
                filter_opcode.as_deref(),
                direction,
                sort,
                reverse,
                limit,
                output,
            );
        }
        Some(Commands::Fragments {
            direction,
            sort,
            reverse,
            limit,
            output,
        }) => {
            output_fragments(&packets, direction, sort, reverse, limit, output);
        }
        Some(Commands::Weenies {
            object_id,
            name,
            sort,
            reverse,
            limit,
            output,
        }) => {
            output_weenies(
                &weenie_db,
                object_id.as_deref(),
                name.as_deref(),
                sort,
                reverse,
                limit,
                output,
            );
        }
        Some(Commands::Summary) => {
            print_summary(&packets, &messages, &weenie_db);
        }
        Some(Commands::Tui) => {
            tui::run_tui(messages, packets)?;
        }
        None => {
            // Default: output messages as JSONL
            for message in &messages {
                println!("{}", serde_json::to_string(&message)?);
            }
        }
    }

    Ok(())
}
