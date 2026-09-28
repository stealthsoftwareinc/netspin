//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::fmt;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Sub;
use core::time::Duration;
use std::time::SystemTime;

const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// A moment on the simulation clock.
///
/// This is analogous to [`std::time::Instant`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Moment {
  nanos: u64,
}

impl Moment {
  /// The starting moment of the simulation clock.
  pub const ZERO: Self = Self { nanos: 0 };

  pub(crate) fn from_nanos(nanos: u64) -> Self {
    Self { nanos }
  }

  /// Returns how far this moment is past the starting moment.
  #[must_use]
  pub fn to_duration(&self) -> Duration {
    Duration::from_nanos(self.nanos)
  }

  /// Returns the real-world time represented by this moment.
  ///
  /// `epoch` is the real-world time represented by [`Moment::ZERO`].
  ///
  /// Panics if the resulting real-world time is out of range.
  ///
  /// [`Moment::ZERO`]: crate::Moment::ZERO
  #[must_use]
  pub fn to_system_time(&self, epoch: SystemTime) -> SystemTime {
    epoch
      .checked_add(self.to_duration())
      .expect("netspin_core::Moment::to_system_time(): Overflow.")
  }
}

/// Displays this [`Moment`] as the exact number of seconds since the
/// start of the simulation, with no unnecessary leading or trailing
/// zeros.
///
/// [`Moment`]: crate::Moment
impl fmt::Display for Moment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let seconds = self.nanos / NANOS_PER_SECOND;
    let mut fraction = self.nanos % NANOS_PER_SECOND;
    if fraction == 0 {
      return write!(f, "{seconds}");
    }
    let mut width = 9;
    while fraction.is_multiple_of(10) {
      fraction /= 10;
      width -= 1;
    }
    write!(f, "{seconds}.{fraction:0width$}")
  }
}

impl Add<Duration> for Moment {
  type Output = Moment;

  fn add(self, duration: Duration) -> Moment {
    let nanos = u64::try_from(duration.as_nanos())
      .ok()
      .and_then(|nanos| self.nanos.checked_add(nanos))
      .expect("Clock overflow");
    Moment::from_nanos(nanos)
  }
}

impl AddAssign<Duration> for Moment {
  fn add_assign(&mut self, duration: Duration) {
    *self = *self + duration;
  }
}

impl Sub for Moment {
  type Output = Duration;

  fn sub(self, rhs: Moment) -> Duration {
    let nanos = self
      .nanos
      .checked_sub(rhs.nanos)
      .expect("Moment subtraction would be negative.");
    Duration::from_nanos(nanos)
  }
}

#[cfg(test)]
mod tests;
