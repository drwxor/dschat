use crate::client::connection;
use rustyline::{DefaultEditor, ExternalPrinter};
use std::error::Error;
use std::io::{self, Write};
use tokio::sync::mpsc;

pub struct Prompt {
    editor: DefaultEditor,
    connection: connection::Connection,
    prompt: String,
}

impl Prompt {
    pub fn new(
        prompt: String,
        connection: connection::Connection,
        mut messages: mpsc::UnboundedReceiver<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut editor = DefaultEditor::new()?;
        let mut printer = editor.create_external_printer()?;

        tokio::spawn(async move {
            while let Some(message) = messages.recv().await {
                let _ = printer.print(message);
            }
        });

        Ok(Self {
            editor,
            connection,
            prompt,
        })
    }

    pub async fn process(&mut self) -> Result<bool, Box<dyn Error>> {
        let line = match self.editor.readline(&self.prompt) {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted) => return Ok(true),
            Err(rustyline::error::ReadlineError::Eof) => return Ok(false),
            Err(error) => return Err(error.into()),
        };

        print!("\x1b[1A\r\x1b[2K");
        io::stdout().flush()?;

        let message = line.trim();

        if message.is_empty() {
            return Ok(true);
        }

        if message == "/exit" {
            return Ok(false);
        }

        if message == "/clear" {
            print!("\x1b[2J\x1b[H");
            io::stdout().flush()?;
            return Ok(true);
        }

        self.editor.add_history_entry(message)?;
        self.connection.send(message).await?;

        Ok(true)
    }
}
