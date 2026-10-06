use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub struct Connection {
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl Connection {
    pub async fn new(
        server: String,
        name: String,
    ) -> Result<(Self, mpsc::UnboundedReceiver<String>), Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(server).await?;
        let (reader, mut writer) = stream.into_split();

        writer.write_all(name.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        let (sender, receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);

            loop {
                let mut message = String::new();

                match reader.read_line(&mut message).await {
                    Ok(0) => break,
                    Ok(_) => {
                        if sender.send(message.trim_end().to_string()).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        eprintln!("connection error: {error}");
                        break;
                    }
                }
            }
        });

        Ok((Self { writer }, receiver))
    }

    pub async fn send(&mut self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;

        Ok(())
    }
}
