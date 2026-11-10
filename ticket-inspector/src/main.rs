use clap::Parser;
use anyhow::{Context, Result};
use base32::Alphabet;
use std::str;
#[derive(Parser, Debug)]

struct Ticket {
content: String
}


fn main() -> Result<()> {
let cli = Ticket::parse();
let boundaries = cli.content.floor_char_boundary(52);
let node = &cli.content[..boundaries];
let ascii_code = base32::decode(Alphabet::Z, node).context("failed to decode the zbase32")?;
let nodeu = str::from_utf8(&ascii_code)?;
println!("Ticket node is {} which is the ip {}", node, nodeu);

Ok (())
}