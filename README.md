# halo-cli

CLI frontend for the [`halo`](../halod) MoQ video streaming crate.

## Usage

```bash
# Publish a video file (prints a join ticket to stderr)
cargo run -- publish path/to/video.mp4

# Subscribe using a join ticket (streams fMP4 to stdout, e.g. piped to ffplay)
cargo run -- subscribe <TICKET> | ffplay -i pipe:0

# Inspect a join ticket (prints its version, node ID, and IP address)
cargo run -- inspect <TICKET>
```
