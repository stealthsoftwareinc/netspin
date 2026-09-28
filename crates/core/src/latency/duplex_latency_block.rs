//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Duplex;
use crate::Spawn;
use crate::latency::LatencyBlock;

/// Configures buffered latency independently in both directions of a
/// [`Duplex`] endpoint.
///
/// The transmit and receive directions each use a separate
/// [`LatencyBlock`].
///
/// [`Duplex`]: crate::Duplex
/// [`LatencyBlock`]: crate::latency::LatencyBlock
pub struct DuplexLatencyBlock<T, R = T> {
  tx: LatencyBlock<T>,
  rx: LatencyBlock<R>,
}

impl<T, R> Clone for DuplexLatencyBlock<T, R> {
  fn clone(&self) -> Self {
    Self {
      tx: self.tx.clone(),
      rx: self.rx.clone(),
    }
  }
}

impl<T, R> DuplexLatencyBlock<T, R> {
  /// Creates a duplex latency block with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      tx: LatencyBlock::new(),
      rx: LatencyBlock::new(),
    }
  }

  /// Sets the latency block for transmitted messages.
  ///
  /// The default is [`LatencyBlock::new()`].
  ///
  /// [`LatencyBlock::new()`]: LatencyBlock::new
  #[must_use]
  pub fn tx(mut self, tx: LatencyBlock<T>) -> Self {
    self.tx = tx;
    self
  }

  /// Sets the latency block for received messages.
  ///
  /// The default is [`LatencyBlock::new()`].
  ///
  /// [`LatencyBlock::new()`]: LatencyBlock::new
  #[must_use]
  pub fn rx(mut self, rx: LatencyBlock<R>) -> Self {
    self.rx = rx;
    self
  }
}

impl<T> DuplexLatencyBlock<T> {
  /// Sets the same latency block for transmitted and received messages.
  ///
  /// The default is [`LatencyBlock::new()`] for both directions.
  ///
  /// [`LatencyBlock::new()`]: LatencyBlock::new
  #[must_use]
  pub fn both(mut self, block: LatencyBlock<T>) -> Self {
    self.tx = block.clone();
    self.rx = block;
    self
  }
}

impl<T, R> Default for DuplexLatencyBlock<T, R> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: 'static, R: 'static, E: Error + 'static>
  Attach<(Duplex<T, R, E>, Duplex<R, T, E>)>
  for DuplexLatencyBlock<T, R>
{
  type Surface = ();

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: (Duplex<T, R, E>, Duplex<R, T, E>),
  ) -> Self::Surface {
    let (a, b) = target;
    self.tx.attach(spawner, (b.rx, a.tx));
    self.rx.attach(spawner, (a.rx, b.tx));
  }
}

impl<T: 'static, R: 'static, E: Error + 'static> Attach<Duplex<T, R, E>>
  for DuplexLatencyBlock<T, R>
{
  type Surface = Duplex<T, R, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Duplex<T, R, E>,
  ) -> Self::Surface {
    let (a, b) = Duplex::pair();
    Self::attach(self, spawner, (target, a));
    b
  }
}

#[cfg(test)]
mod tests;
