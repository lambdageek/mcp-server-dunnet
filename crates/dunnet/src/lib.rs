//! A wrapper around Emacs dunnet
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::io::Write;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::task;

mod mcp_server;
mod toggle;

pub struct Dunnet {
    process: Child,
}

/// A command that we send to the dunnet world
#[derive(Debug, Clone)]
pub enum DunnetInput {
    /// Tell the game to quit
    Quit,
    /// Send a command to the game and get a response
    Command(String),
}

#[derive(Debug)]
pub enum DunnetResponse {
    /// The Emacs process has finished processing the input.  Carries any accumulated output
    Done(Vec<String>),
    /// Lines of output, the last line with "\n>" is not included
    Output(Vec<String>),
}

#[derive(Debug)]
pub struct DunnetRepl {
    /// An event that indicates when we're done sending input to the Emacs process
    input_done_tx: toggle::ToggleSender,
    // a writer for sending input to the Emacs process
    child_in: io::BufWriter<tokio::process::ChildStdin>,
    // a channel receiver for receiving responses from the Emacs process
    response_rx: futures::channel::mpsc::Receiver<DunnetResponse>,
}

impl Dunnet {
    pub fn new() -> Self {
        let path = "/Applications/Emacs.app/Contents/MacOS/emacs-nw";
        let args = ["-q", "-batch", "-l", "dunnet"];
        let process = Command::new(path)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start Emacs process");
        Dunnet { process }
    }

    pub fn repl(self) -> DunnetRepl {
        DunnetRepl::new(self.process)
    }

    pub async fn play_stdio(self) {
        let mut stdin = io::BufReader::new(io::stdin());
        let mut repl = self.repl();
        let mut response: DunnetResponse = repl.game_start().await;
        loop {
            match response {
                DunnetResponse::Done(lines) => {
                    for line in lines {
                        println!("O: {}", line);
                    }
                    break;
                }
                DunnetResponse::Output(lines) => {
                    for line in lines {
                        println!("O: {}", line);
                    }
                }
            }
            print!("Your command: ");
            std::io::stdout().flush().expect("Failed to flush stdout");
            let mut input = String::new();
            stdin
                .read_line(&mut input)
                .await
                .expect("Failed to read from stdin");
            response = repl.interact(input).await;
        }
    }

    pub async fn mcp_server(&mut self) {
        mcp_server::server_main().await.expect("MCP server failed");
    }
}

impl DunnetRepl {
    pub fn new(mut process: Child) -> Self {
        let (input_done_tx, input_done_rx) = toggle::new_toggle();
        let child_out = io::BufReader::new(process.stdout.take().unwrap());
        let (responses_tx, responses_rx) = futures::channel::mpsc::channel(100);

        let repl = DunnetRepl {
            input_done_tx,
            child_in: io::BufWriter::new(process.stdin.take().unwrap()),
            response_rx: responses_rx,
        };
        // Spawn a task that collects output from the Emacs process and puts it into the response channel
        task::spawn(OutputHandler::new(input_done_rx, responses_tx).task(child_out));

        // Spawn a task that waits for the process to end
        task::spawn(async move {
            process.wait().await.expect("wait failed");
        });

        repl
    }

    async fn response(&mut self) -> DunnetResponse {
        self.response_rx
            .next()
            .await
            .unwrap_or(DunnetResponse::Done(Vec::new()))
    }

    /// Wait for the initial game start response (and prompt)
    pub async fn game_start(&mut self) -> DunnetResponse {
        self.response().await
    }

    pub async fn quit(&mut self) -> DunnetResponse {
        self.input_done_tx.toggle();
        self.response().await
    }

    pub async fn interact(&mut self, mut input: String) -> DunnetResponse {
        if !input.ends_with("\n") {
            input.push('\n');
        }
        self.child_in
            .write_all(input.as_bytes())
            .await
            .expect("Failed to write to Emacs stdin");
        self.child_in
            .flush()
            .await
            .expect("Failed to flush Emacs stdin");
        self.response().await
    }
}

/// Do something with the output of the Emacs process
struct OutputHandler {
    // an event that indicates when we're done sending input to the Emacs process, and we can stop waiting for output
    input_done: toggle::ToggleReceiver,
    // a channel sender for sending responses back to the DunnetRepl
    responses_tx: futures::channel::mpsc::Sender<DunnetResponse>,
}

impl OutputHandler {
    fn new(
        input_done: toggle::ToggleReceiver,
        responses_tx: futures::channel::mpsc::Sender<DunnetResponse>,
    ) -> Self {
        OutputHandler {
            input_done,
            responses_tx,
        }
    }

    async fn task(mut self, mut child_out: io::BufReader<tokio::process::ChildStdout>) {
        let mut accumulated_bytes = Vec::new();
        let mut child_done = false;
        while !child_done {
            let mut bytes: Vec<u8> = Vec::new();

            let flush_accumulated_output = futures::select! {
                n_bytes = child_out.read_until(b'>', &mut bytes).fuse() => {
                    match n_bytes {
                        Ok(0) => {
                            // EOF reached, exit the loop; flush accumulated bytes
                            child_done = true;
                            true
                        },
                        Ok(_count) => {
                            let reached_prompt = if bytes.ends_with(b"\n>") {
                                // Remove the trailing ">"
                                bytes.truncate(bytes.len() - 1);
                                eprintln!("Reached prompt, flushing accumulated bytes");
                                true
                            } else {
                                // No prompt found, accumulate what we have so far
                                // and continue reading
                                false
                            };
                            accumulated_bytes.extend(bytes);
                            reached_prompt
                        },
                        Err(err) => {
                            eprintln!("Error reading line: {}", err);
                            child_done = true;
                            // flush accumulated bytes
                            true
                        },
                    }
                }
                _ = self.input_done.wait() => { child_done = true; true }
            };
            if flush_accumulated_output && !accumulated_bytes.is_empty() {
                // We have reached the prompt, process the accumulated bytes
                let output = String::from_utf8_lossy(&accumulated_bytes).to_string();
                let lines: Vec<String> = output.lines().map(|line| line.to_string()).collect();
                let response = if child_done {
                    DunnetResponse::Done(lines)
                } else {
                    DunnetResponse::Output(lines)
                };
                self.responses_tx
                    .try_send(response)
                    .expect("Failed to send Output response");
                accumulated_bytes.clear();
            }
        }
        let last_lines = if !accumulated_bytes.is_empty() {
            // If we have any remaining accumulated bytes, send them as a Done response
            let output = String::from_utf8_lossy(&accumulated_bytes).to_string();
            output.lines().map(|line| line.to_string()).collect()
        } else {
            Vec::new()
        };
        self.responses_tx
            .try_send(DunnetResponse::Done(last_lines))
            .expect("Failed to send Done response");
    }
}
