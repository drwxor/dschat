use clap::Parser;
use dschat::client;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum CliError {
    UnknownName,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownName => write!(f, "unknown name"),
        }
    }
}

impl Error for CliError {}

#[derive(Parser, Debug)]
#[command(name = "dchat")]
struct Args {
    #[arg(long)]
    name: Option<String>,

    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    ip: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let Some(name) = args.name else {
        println!("please tell me who you are");
        println!("dchat --name <YOUR NAME>");
        return Err(CliError::UnknownName.into());
    };

    println!("setting name to '{name}'");

    let ip = args.ip;
    let port = args.port;
    let ip_port = format!("{ip}:{port}");

    println!("connecting to server '{ip_port}'");

    let (connection, messages) = client::connection::Connection::new(ip_port, name.clone()).await?;

    let mut prompt = client::prompt::Prompt::new(format!("{name}> "), connection, messages)?;

    loop {
        if !prompt.process().await? {
            break;
        }
    }

    Ok(())
}
