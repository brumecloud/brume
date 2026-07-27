use std::{
    io::{self, IsTerminal, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone)]
struct Status {
    step: usize,
    message: String,
}

pub struct Progress {
    total: usize,
    interactive: bool,
    status: Arc<Mutex<Status>>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Progress {
    pub fn start(total: usize, message: impl Into<String>) -> Self {
        assert!(total > 0, "progress must contain at least one step");
        let status = Arc::new(Mutex::new(Status {
            step: 1,
            message: message.into(),
        }));
        let stopped = Arc::new(AtomicBool::new(false));
        let interactive = io::stderr().is_terminal();

        let worker = if interactive {
            let status = Arc::clone(&status);
            let stopped = Arc::clone(&stopped);
            Some(thread::spawn(move || {
                let mut frame = 0;
                while !stopped.load(Ordering::Relaxed) {
                    let current = status.lock().expect("progress status poisoned").clone();
                    eprint!(
                        "\r\x1b[2K{}",
                        tty_line(
                            FRAMES[frame % FRAMES.len()],
                            current.step,
                            total,
                            &current.message
                        )
                    );
                    let _ = io::stderr().flush();
                    frame += 1;
                    thread::park_timeout(Duration::from_millis(80));
                }
            }))
        } else {
            let current = status.lock().expect("progress status poisoned");
            eprintln!("{}", plain_line(current.step, total, &current.message));
            None
        };

        Self {
            total,
            interactive,
            status,
            stopped,
            worker,
        }
    }

    pub fn advance(&mut self, message: impl Into<String>) {
        let mut status = self.status.lock().expect("progress status poisoned");
        assert!(
            status.step < self.total,
            "progress advanced past final step"
        );
        status.step += 1;
        status.message = message.into();
        if !self.interactive {
            eprintln!("{}", plain_line(status.step, self.total, &status.message));
        }
        drop(status);
        if let Some(worker) = &self.worker {
            worker.thread().unpark();
        }
    }

    pub fn finish(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
        if self.interactive {
            eprint!("\r\x1b[2K");
            let _ = io::stderr().flush();
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.stop();
    }
}

fn tty_line(frame: &str, step: usize, total: usize, message: &str) -> String {
    format!("{frame} [{step}/{total}] {message}")
}

fn plain_line(step: usize, total: usize, message: &str) -> String {
    format!("[{step}/{total}] {message}")
}

#[cfg(test)]
mod tests {
    use super::{plain_line, tty_line};

    #[test]
    fn formats_plain_progress() {
        assert_eq!(plain_line(2, 4, "Packaging files"), "[2/4] Packaging files");
    }

    #[test]
    fn formats_interactive_progress() {
        assert_eq!(
            tty_line("⠋", 3, 5, "Uploading plan"),
            "⠋ [3/5] Uploading plan"
        );
    }
}
