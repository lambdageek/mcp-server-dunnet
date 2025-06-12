//! A wrapper around Emacs dunnet
use std::ops::Drop;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::process::{Command, Child};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::task;

pub struct Dunnet {
    process: Child,
}

#[derive(Debug)]
struct SharedState {
    done: AtomicBool,
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
        let shared_state = Arc::new(SharedState {
            done: AtomicBool::new(false),
        });
        let shared_state_clone = Arc::clone(&shared_state);
        task::spawn(async move {
            let shared_state = &*shared_state_clone;
            let mut lines = child_out.lines();
            while let Some(line) = lines.next_line().await.unwrap() {
                let line = line.strip_prefix(">").unwrap_or(&line).trim();
                println!("Output: {}", line);
            }
            shared_state.done.store(true, Ordering::Release);
        });
        let shared_state = &*shared_state;
        let mut stdin = io::BufReader::new(io::stdin());
        loop {
            if shared_state.done.load(Ordering::Acquire) {
                break;
            }
            let mut input = String::new();
            let len = stdin.read_line(&mut input).await.expect("Failed to read line");
            self.process.stdin.as_mut().expect("Failed to get stdin")
                .write_all(input.as_bytes()).await.expect("Failed to write to Emacs stdin");
            if len == 0 {
                // EOF reached, exit the loop
                break;
            }

        }
        self.process.stdin.take().expect("Failed to get stdin").shutdown().await.expect("Failed to shutdown Emacs stdin");

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
}