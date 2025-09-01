use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server host, default localhost
    #[arg(long, env = "HOST", default_value = "localhost")]
    pub host: String,

    /// Server port, default 8000
    #[arg(long, env = "PORT", default_value_t = 8000)]
    pub port: u16,

    /// Database url
    #[arg(
        long,
        env = "DB_URL",
        default_value = "postgresql://postgres:password@localhost:5432/postgres"
    )]
    pub db_url: String,

    /// Redis url, default redis://127.0.0.1:6379
    #[arg(long, env = "REDIS_URL", default_value = "redis://127.0.0.1:6379")]
    pub redis_url: String,

    /// Frontend origin, default http://localhost:5173
    #[arg(long, env = "FRONTEND_ORIGIN", default_value = "http://localhost:5173")]
    pub frontend_origin: String,
}
