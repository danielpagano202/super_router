use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
use std::process::Stdio;
pub struct AsyncProcess {
    //child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl AsyncProcess {
    pub fn new(command: &str, args: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let input = child.stdin.take().ok_or("Failed to open stdin")?;
        let raw_stdout = child.stdout.take().ok_or("Failed to open stdout")?;

        let output = tokio::io::BufReader::new(raw_stdout); 

        Ok(AsyncProcess { input, output })
    }

    pub async fn write(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.input.write_all(data.as_bytes()).await?;
        self.input.flush().await?; // 2. Added flush so the child process actually receives it
        Ok(())
    }

    pub async fn read(&mut self) -> Result<String, std::io::Error> {
        let mut response_line = String::new();
        self.output.read_line(&mut response_line).await?;
        Ok(response_line) // 3. Removed trailing semicolon so it correctly returns the value
    }
}
