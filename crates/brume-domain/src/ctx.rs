//! What a core reads from the outside world.

use crate::time::Timestamp;

/// The clock and the randomness of a core. The shells implement it with the system clock and
/// the operating system's generator; tests, with a manual clock and a seeded generator, so that
/// a run replays exactly.
pub trait Ctx {
    /// The current time.
    fn now(&self) -> Timestamp;

    /// Fills `buf` with random bytes. Outside tests, they come from a cryptographically secure
    /// generator: login tokens are drawn here.
    fn fill_random(&mut self, buf: &mut [u8]);
}
