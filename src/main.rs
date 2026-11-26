use anyhow::Result;
use base32::Alphabet;
use clap::{Parser, Subcommand};
use halo::{Keypair, Publisher, Ticket, decode_ticket, parse_ticket, subscribe};
use std::net::SocketAddr;
use tracing::{Level, event};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "halo-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish a video from a file, using MoQ
    Publish {
        video_path: String,
        /// Optional external discovery server address (e.g. [2001:db8::1]:4444)
        #[arg(long)]
        discovery: Option<SocketAddr>,
    },

    /// Subscribe to a MoQ stream using a ticket
    Subscribe {
        ticket: String,
        /// Optional external discovery server address (e.g. [2001:db8::1]:4444)
        #[arg(long)]
        discovery: Option<SocketAddr>,
    },

    /// Decode and inspect a HALO join ticket
    Inspect { ticket: String },
}

fn format_ticket(ticket: &Ticket) -> String {
    let node_id = base32::encode(Alphabet::Z, &ticket.node_id);
    format!(
        "Version: {}\nNode ID: {}\nIP Address: {}",
        ticket.version, node_id, ticket.ip
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Commands::Inspect { ticket } = cli.command {
        let decoded = decode_ticket(&ticket)?;
        println!("{}", format_ticket(&decoded));
        return Ok(());
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    event!(Level::INFO, "starting halo-cli {}", env!("CARGO_PKG_VERSION"));

    let keypair = Keypair::new()?;
    event!(Level::INFO, "pubkey for this node is {keypair}");

    match cli.command {
        Commands::Publish {
            video_path,
            discovery,
        } => {
            eprintln!("Publishing {video_path}");
            let publisher = Publisher::bind(&video_path, keypair.node_id(), discovery).await?;
            eprintln!("Publisher ticket: {}", publisher.ticket());
            tokio::select! {
                res = publisher.run() => res?,
                _ = tokio::signal::ctrl_c() => {}
            }
        }
        Commands::Subscribe { ticket, discovery } => {
            // Printing anything to stdout can corrupt the pipe to ffplay
            eprintln!("Subscribing to {ticket}");
            let addr = parse_ticket(&ticket)?;
            tokio::select! {
                res = subscribe(addr, tokio::io::stdout(), discovery) => res?,
                _ = tokio::signal::ctrl_c() => {}
            }
        }
        Commands::Inspect { .. } => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo::encode_ticket;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn parses_subcommands() {
        let publish = Cli::try_parse_from(["halo-cli", "publish", "video.mp4"]).unwrap();
        assert!(matches!(
            publish.command,
            Commands::Publish { video_path, discovery: None } if video_path == "video.mp4"
        ));

        let publish_disc = Cli::try_parse_from([
            "halo-cli",
            "publish",
            "video.mp4",
            "--discovery",
            "[::1]:4444",
        ])
        .unwrap();
        assert!(matches!(
            publish_disc.command,
            Commands::Publish { video_path, discovery: Some(addr) }
                if video_path == "video.mp4" && addr == "[::1]:4444".parse::<SocketAddr>().unwrap()
        ));

        let subscribe = Cli::try_parse_from(["halo-cli", "subscribe", "abc"]).unwrap();
        assert!(matches!(
            subscribe.command,
            Commands::Subscribe { ticket, discovery: None } if ticket == "abc"
        ));

        let inspect = Cli::try_parse_from(["halo-cli", "inspect", "abc"]).unwrap();
        assert!(matches!(
            inspect.command,
            Commands::Inspect { ticket } if ticket == "abc"
        ));
    }

    #[test]
    fn formats_decoded_ipv4_and_ipv6_tickets() {
        let node_id = [7u8; 32];
        let expected_node_id = base32::encode(Alphabet::Z, &node_id);

        let v4_ticket = encode_ticket(&node_id, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)));
        let decoded_v4 = decode_ticket(&v4_ticket).unwrap();
        assert_eq!(
            format_ticket(&decoded_v4),
            format!("Version: 1\nNode ID: {expected_node_id}\nIP Address: 192.168.1.20")
        );

        let v6_ticket = encode_ticket(&node_id, IpAddr::V6(Ipv6Addr::LOCALHOST));
        let decoded_v6 = decode_ticket(&v6_ticket).unwrap();
        assert_eq!(
            format_ticket(&decoded_v6),
            format!("Version: 1\nNode ID: {expected_node_id}\nIP Address: ::1")
        );
    }
}
