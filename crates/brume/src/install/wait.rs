//! The steps that wait for the admin: the Tailscale login, HTTPS on the tailnet, the engine's
//! start. They print a link, poll, remind, and give up after a while. Their clock is injected,
//! so that the tests drive the time.

use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// The longest wait for the admin: then the install stops, and the same command resumes it.
pub const WAIT_LIMIT: Duration = Duration::from_secs(15 * 60);

/// How often the link is printed again while the install waits.
pub const REMIND_EVERY: Duration = Duration::from_secs(2 * 60);

/// How often the host is asked whether the admin is done.
pub const POLL_EVERY: Duration = Duration::from_secs(2);

/// The longest link printed: a longer one is not Tailscale's.
const MAX_LINK_BYTES: usize = 2048;

/// The time of the install.
pub trait Clock {
    /// The time since a fixed instant.
    fn now(&self) -> Duration;
    fn sleep(&self, duration: Duration);
}

/// The time of this machine.
struct SystemClock {
    start: Instant,
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// The clock and the terminal of the steps that talk to the admin.
pub struct Console {
    clock: Box<dyn Clock>,
    out: RefCell<Box<dyn Write>>,
}

impl Console {
    pub fn new(clock: Box<dyn Clock>, out: Box<dyn Write>) -> Rc<Self> {
        Rc::new(Self {
            clock,
            out: RefCell::new(out),
        })
    }

    /// Standard output and this machine's time.
    pub fn real() -> Rc<Self> {
        Self::new(
            Box::new(SystemClock {
                start: Instant::now(),
            }),
            Box::new(std::io::stdout()),
        )
    }

    /// Prints `line`, indented under the step. A terminal that is gone does not stop the
    /// install.
    pub fn say(&self, line: &str) {
        let mut out = self.out.borrow_mut();
        let _ = writeln!(out, "  {line}");
        let _ = out.flush();
    }

    pub fn now(&self) -> Duration {
        self.clock.now()
    }

    pub fn sleep(&self, duration: Duration) {
        self.clock.sleep(duration);
    }
}

/// What the admin is asked to do: a sentence, then the link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub action: &'static str,
    pub link: String,
}

/// What a poll found.
pub enum Poll<T> {
    Ready(T),
    /// Still waiting, for the admin to follow the link if there is one.
    Waiting(Option<Prompt>),
}

/// The admin did not act in time. The install stops; the same command resumes it.
#[derive(Debug)]
pub struct TimedOut(pub String);

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TimedOut {}

/// Whether `error` is a wait that timed out.
pub fn timed_out(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| cause.is::<TimedOut>())
}

/// Polls `poll` until it is ready, `WAIT_LIMIT` at most. Prints the prompt as soon as it is
/// known, again when it changes, and every `REMIND_EVERY`. `what` names what the install waits
/// for, such as "the Tailscale login".
pub fn wait_for<T>(
    console: &Console,
    what: &str,
    mut poll: impl FnMut() -> anyhow::Result<Poll<T>>,
) -> anyhow::Result<T> {
    let start = console.now();
    // The prompt printed last, and when.
    let mut shown: Option<(Prompt, Duration)> = None;
    let mut first = true;
    loop {
        let prompt = match poll()? {
            Poll::Ready(value) => return Ok(value),
            Poll::Waiting(prompt) => prompt,
        };
        if first && prompt.is_none() {
            console.say(&format!(
                "Waiting for {what}, {} minutes at most…",
                WAIT_LIMIT.as_secs() / 60
            ));
        }
        first = false;
        let elapsed = console.now().saturating_sub(start);
        if let Some(prompt) = prompt {
            match &shown {
                Some((old, at)) if *old == prompt => {
                    if elapsed.saturating_sub(*at) >= REMIND_EVERY {
                        let left = WAIT_LIMIT.saturating_sub(elapsed).as_secs().div_ceil(60);
                        console.say(&format!(
                            "Still waiting for {what}, {left} min left: {}",
                            prompt.link
                        ));
                        shown = Some((prompt, elapsed));
                    }
                }
                _ => {
                    console.say(prompt.action);
                    console.say(&format!("  {}", prompt.link));
                    console.say(&format!(
                        "Waiting for {what}, {} minutes at most…",
                        WAIT_LIMIT.as_secs() / 60
                    ));
                    shown = Some((prompt, elapsed));
                }
            }
        }
        if elapsed >= WAIT_LIMIT {
            let link = shown
                .map(|(prompt, _)| format!(" ({})", prompt.link))
                .unwrap_or_default();
            return Err(TimedOut(format!(
                "Gave up waiting for {what} after {} minutes{link}.",
                WAIT_LIMIT.as_secs() / 60
            ))
            .into());
        }
        console.sleep(POLL_EVERY);
    }
}

/// `text` if it is an https link that is safe to print: printable ASCII, bounded.
pub fn link(text: &str) -> Option<String> {
    let valid = text.starts_with("https://")
        && text.len() > "https://".len()
        && text.len() <= MAX_LINK_BYTES
        && text.bytes().all(|b| b.is_ascii_graphic());
    valid.then(|| text.to_owned())
}

#[cfg(test)]
pub mod fake {
    //! A clock that moves only when the install sleeps, and a terminal that keeps its lines.

    use std::cell::{Cell, RefCell};
    use std::io::Write;
    use std::rc::Rc;
    use std::time::Duration;

    use super::{Clock, Console};

    struct FakeClock(Rc<Cell<Duration>>);

    impl Clock for FakeClock {
        fn now(&self) -> Duration {
            self.0.get()
        }

        fn sleep(&self, duration: Duration) {
            self.0.set(self.0.get() + duration);
        }
    }

    struct Lines(Rc<RefCell<Vec<u8>>>);

    impl Write for Lines {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// A console, what it printed, and its time.
    pub struct FakeConsole {
        pub console: Rc<Console>,
        printed: Rc<RefCell<Vec<u8>>>,
        time: Rc<Cell<Duration>>,
    }

    impl FakeConsole {
        pub fn new() -> Self {
            let printed = Rc::new(RefCell::new(Vec::new()));
            let time = Rc::new(Cell::new(Duration::ZERO));
            Self {
                console: Console::new(
                    Box::new(FakeClock(Rc::clone(&time))),
                    Box::new(Lines(Rc::clone(&printed))),
                ),
                printed,
                time,
            }
        }

        pub fn printed(&self) -> String {
            String::from_utf8(self.printed.borrow().clone()).expect("UTF-8")
        }

        pub fn elapsed(&self) -> Duration {
            self.time.get()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::fake::FakeConsole;
    use super::{Poll, Prompt, link, timed_out, wait_for};

    #[test]
    fn reminds_every_two_minutes_then_times_out() {
        let fake = FakeConsole::new();
        let error = wait_for::<()>(&fake.console, "the Tailscale login", || {
            Ok(Poll::Waiting(Some(Prompt {
                action: "Open this link:",
                link: "https://login.tailscale.com/a/abc".to_owned(),
            })))
        })
        .expect_err("times out");
        assert!(timed_out(&error));
        assert_eq!(
            error.to_string(),
            "Gave up waiting for the Tailscale login after 15 minutes \
             (https://login.tailscale.com/a/abc)."
        );
        let printed = fake.printed();
        assert_eq!(
            printed.matches("https://login.tailscale.com/a/abc").count(),
            8
        );
        assert!(printed.contains("Still waiting for the Tailscale login, 13 min left"));
        assert!(fake.elapsed() >= Duration::from_secs(15 * 60));
        assert!(fake.elapsed() < Duration::from_secs(15 * 60 + 5));
    }

    #[test]
    fn a_new_link_is_printed_at_once() {
        let fake = FakeConsole::new();
        let mut polls = 0;
        let done = wait_for(&fake.console, "the login", || {
            polls += 1;
            Ok(match polls {
                1 => Poll::Waiting(None),
                2 => Poll::Waiting(Some(Prompt {
                    action: "Open:",
                    link: "https://a.example/1".to_owned(),
                })),
                3 => Poll::Waiting(Some(Prompt {
                    action: "Open:",
                    link: "https://a.example/2".to_owned(),
                })),
                _ => Poll::Ready(polls),
            })
        })
        .expect("ready");
        assert_eq!(done, 4);
        let printed = fake.printed();
        assert!(printed.contains("https://a.example/1"));
        assert!(printed.contains("https://a.example/2"));
    }

    #[test]
    fn prints_only_clean_links() {
        assert_eq!(
            link("https://login.tailscale.com/a/1").as_deref(),
            Some("https://login.tailscale.com/a/1")
        );
        for text in [
            "http://login.tailscale.com/a/1",
            "https://",
            "https://a\u{1b}[31m",
            "https://a b",
        ] {
            assert_eq!(link(text), None, "{text:?}");
        }
    }
}
