//! A wrapper around Emacs dunnet
use futures::stream::StreamExt;
use futures::future::FutureExt;
use std::sync::Arc;
use tokio::process::{Command, Child};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::task;

mod toggle;
mod mcp_server;

pub struct Dunnet {
    process: Child,
}

#[derive(Debug, Clone)]
struct SharedState {
    dunnet_done: toggle::ToggleReceiver,
    input_done: toggle::ToggleReceiver,
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

    pub async fn repl(&mut self) {
        let child_out = io::BufReader::new(self.process.stdout.take().unwrap());
        let (dunnet_done_tx, dunnet_done_rx) = toggle::new_toggle();
        let (input_done_tx, input_done_rx) = toggle::new_toggle();
        let shared_state = Arc::new(SharedState {
            dunnet_done: dunnet_done_rx,
            input_done: input_done_rx,
        });
        task::spawn(OutputHandler::new(shared_state.clone(), dunnet_done_tx).task(child_out));
        InputHandler::new(shared_state.clone(), input_done_tx).task(io::BufWriter::new(self.process.stdin.take().unwrap())).await;
        println!("REPL session ended.");
        match self.process.wait().await {
            Ok(status) => {
            println!("Emacs process exited with status: {}", status);
            },
            Err(err) => {
            eprintln!("Emacs process terminated unexpectedly: {}", err);
            }
        }
    }

    pub async fn mcp_server(&mut self) {
        mcp_server::server_main().await.expect("MCP server failed");
    }
}

/// Do something with the output of the Emacs process
struct OutputHandler {
    shared_state: Arc<SharedState>,
    dunnet_done_tx: toggle::ToggleSender,
}

impl OutputHandler {
    fn new(shared_state: Arc<SharedState>, dunnet_done_tx: toggle::ToggleSender) -> Self {
        OutputHandler { shared_state, dunnet_done_tx }
    }

    async fn task(mut self, child_out: io::BufReader<tokio::process::ChildStdout>) {
        let mut lines = tokio_stream::wrappers::LinesStream::new(child_out.lines()).fuse();
        let mut done = false;
        while !done {
            let mut input_done_fut = self.shared_state.input_done.wait();

            futures::select! {
                line_opt = lines.next() => {
                    match line_opt {
                        Some(Ok(line)) => {
                            let line = line.strip_prefix(">").unwrap_or(&line).trim();
                            println!("Output: {}", line);
                        },
                        Some(Err(err)) => {
                            eprintln!("Error reading line: {}", err);
                            done = true;
                        },
                        None => {
                            // EOF reached, exit the loop
                            done = true;
                        }
                    }
                }
                _ = input_done_fut => { done = true; }
            }
        }
        self.dunnet_done_tx.toggle();
    }
}

/// Provide a way to send input to the Emacs process
struct InputHandler {
    shared_state: Arc<SharedState>,
    input_done_tx: toggle::ToggleSender,
}

impl InputHandler {
    fn new(shared_state: Arc<SharedState>, input_done_tx: toggle::ToggleSender) -> Self {
        InputHandler { shared_state, input_done_tx }
    }

    async fn task(mut self, mut child_in: io::BufWriter<tokio::process::ChildStdin>) {
        let shared_state = &*self.shared_state;
        let mut done = false;
        let mut stdin = io::BufReader::new(tokio::io::stdin());
        while !done {
            let mut input = String::new();
            futures::select! {
                _ = shared_state.dunnet_done.wait() => {
                    done = true;
                }
                len = stdin.read_line(&mut input).fuse() => {
                    match len {
                        Ok(0) => {
                            // EOF reached, exit the loop
                            done = true;
                        }
                        Ok(_len) => {
                            // Successfully read input
                            eprintln!("Input: {}", input.trim());
                        }
                        Err(err) => {
                            eprintln!("Error reading from stdin: {}", err);
                            done = true;
                            continue;
                        }
                    }
                    child_in.write_all(input.as_bytes()).await.expect("Failed to write to Emacs stdin");
                    child_in.flush().await.expect("Failed to flush Emacs stdin");
                }
                
            }
        }
        child_in.shutdown().await.expect("Failed to shutdown Emacs stdin");
        self.input_done_tx.toggle();
    }
}