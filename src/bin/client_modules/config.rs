use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Client host, default localhost
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Client port, default 4000
    #[arg(long, default_value_t = 4000)]
    pub port: u16,

    /// Server host, default localhost
    #[arg(long, default_value = "localhost")]
    pub server: String,

    /// Server port, default 8000
    #[arg(long, default_value_t = 8000)]
    pub server_port: u16,
}
