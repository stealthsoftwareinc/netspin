//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

/// Configures [`Tx::send_with_options()`] and
/// [`Tx::try_send_with_options()`].
///
/// The default settings are zero latency and no timeout override.
///
/// [`Tx::send_with_options()`]: crate::Tx::send_with_options()
/// [`Tx::try_send_with_options()`]: crate::Tx::try_send_with_options()
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendOptions {
  pub(crate) latency: Duration,
  pub(crate) timeout: Option<Duration>,
}

impl SendOptions {
  /// Creates send options with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      latency: Duration::ZERO,
      timeout: None,
    }
  }

  /// Sets how long a sent message takes to arrive.
  ///
  /// If the message is split, the latency is applied to each piece.
  ///
  /// The default is [`Duration::ZERO`].
  ///
  /// [`Duration::ZERO`]: core::time::Duration::ZERO
  #[must_use]
  pub fn latency(mut self, latency: Duration) -> Self {
    self.latency = latency;
    self
  }

  /// Sets how long [`Tx::send_with_options()`] may wait for enough free
  /// space.
  ///
  /// [`Tx::try_send_with_options()`] ignores this setting because it
  /// never waits.
  ///
  /// By default, the endpoint's sending timeout is used.
  ///
  /// [`Tx::send_with_options()`]: crate::Tx::send_with_options()
  /// [`Tx::try_send_with_options()`]: crate::Tx::try_send_with_options()
  #[must_use]
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = Some(timeout);
    self
  }
}

impl Default for SendOptions {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests;
