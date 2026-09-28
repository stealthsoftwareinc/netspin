//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

/// Configures [`Rx::receive_with_options()`] and
/// [`Rx::try_receive_with_options()`].
///
/// The default has no timeout override.
///
/// [`Rx::receive_with_options()`]: crate::Rx::receive_with_options()
/// [`Rx::try_receive_with_options()`]:
/// crate::Rx::try_receive_with_options()
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiveOptions {
  pub(crate) timeout: Option<Duration>,
}

impl ReceiveOptions {
  /// Creates receive options with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self { timeout: None }
  }

  /// Sets how long [`Rx::receive_with_options()`] may wait for a
  /// message.
  ///
  /// [`Rx::try_receive_with_options()`] ignores this setting because it
  /// never waits.
  ///
  /// By default, the endpoint's receiving timeout is used.
  ///
  /// [`Rx::receive_with_options()`]: crate::Rx::receive_with_options()
  /// [`Rx::try_receive_with_options()`]:
  /// crate::Rx::try_receive_with_options()
  #[must_use]
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = Some(timeout);
    self
  }
}

impl Default for ReceiveOptions {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests;
