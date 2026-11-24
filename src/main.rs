use anyhow::{Context, Result};
use base32::Alphabet;
use clap::{Parser};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Parser, Debug)]
struct Cli { 
    ticket: String,
}

#[derive(Debug)]
pub struct Ticket {
    pub version: u8,
    pub real_node: String,
    pub ip: IpAddr,
}

pub fn decode_ticket(cli: &str) -> Result<Ticket> {
 let bytes = base32::decode(Alphabet::Z,cli).context("Failed to decode ticket")?;
let version = bytes[0];
let node_id: [u8; 32] = bytes[1..33].try_into().context("Failed to extract node ID")?;
let real_node= String::from_utf8(node_id.to_vec()).context("Failed to convert node ID to string")?;
let ipversion = bytes[33];
let ip = match ipversion {
    0x04 => {
    let octets: [u8; 4] = bytes[34..38].try_into()?;
            IpAddr::V4(Ipv4Addr::from(octets))
    }
    0x06 => {
      let octets: [u8; 16] = bytes[34..50].try_into()?;
            IpAddr::V6(Ipv6Addr::from(octets))  
    }
    other => Err(anyhow::anyhow!("Unknown IP version: {}", other))?,
};
    Ok(Ticket { version, real_node, ip })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ticket = decode_ticket(&cli.ticket)?;

    println!("Version: {}", ticket.version);
    println!("Real Node: {}", ticket.real_node);
    println!("IP Address: {}", ticket.ip);

    Ok(())
}




