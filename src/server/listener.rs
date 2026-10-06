use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

pub struct Listener {
    listener: TcpListener,
    messages: broadcast::Sender<String>,
}

impl Listener {
    pub async fn new(ip: String, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("{ip}:{port}")).await?;
        let (messages, _) = broadcast::channel(256);

        Ok(Self { listener, messages })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (stream, address) = self.listener.accept().await?;

            println!("connection from {address}");

            let messages = self.messages.clone();

            tokio::spawn(async move {
                if let Err(err) = Self::handle_client(stream, address, messages).await {
                    eprintln!("client {address}: {err}");
                }
            });
        }
    }

    async fn handle_client(
        stream: TcpStream,
        address: std::net::SocketAddr,
        messages: broadcast::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut receiver = messages.subscribe();

        let mut name = String::new();

        if reader.read_line(&mut name).await? == 0 {
            return Ok(());
        }

        let name = name.trim_end().to_string();

        println!("user '{name}' connected from {address}");

        let _ = messages.send(format!("* {name} joined the chat\n"));

        loop {
            let mut message = String::new();

            tokio::select! {
                result = reader.read_line(&mut message) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            let message = message.trim_end();

                            if message.is_empty() {
                                continue;
                            }

                            let message = format!("{name}: {message}\n");
                            let _ = messages.send(message);
                        }
                        Err(error) => {
                            eprintln!("client {address}: {error}");
                            break;
                        }
                    }
                }

                result = receiver.recv() => {
                    match result {
                        Ok(message) => {
                            if let Err(error) = writer.write_all(message.as_bytes()).await {
                                eprintln!("client {address}: {error}");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }

        println!("user '{name}' disconnected from {address}");
        let _ = messages.send(format!("* {name} left the chat\n"));

        Ok(())
    }
}
