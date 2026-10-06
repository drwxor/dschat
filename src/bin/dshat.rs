use clap::Parser;
use dschat::server;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(name = "dshat")]
struct Args {
    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value = "0.0.0.0")]
    ip: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut listener = server::listener::Listener::new(args.ip, args.port).await?;
    listener.run().await?;
    Ok(())
}
