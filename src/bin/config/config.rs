use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server host, default localhost
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Server port, default 8000
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    /// Database url
    #[arg(long, default_value = "postgresql://postgres:password@localhost:5432/postgres")]
    pub db_url: String,

    /// Redis url, default redis://127.0.0.1:6379
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    pub redis_url: String,
}
