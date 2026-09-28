//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Duplex;
use crate::Spawn;
use crate::loss::LossBlock;

/// Configures loss independently in both directions of a [`Duplex`]
/// endpoint.
///
/// The transmit and receive directions each use a separate
/// [`LossBlock`].
///
/// [`Duplex`]: crate::Duplex
/// [`LossBlock`]: crate::loss::LossBlock
#[derive(Clone)]
pub struct DuplexLossBlock {
  tx: LossBlock,
  rx: LossBlock,
}

impl DuplexLossBlock {
  /// Creates a duplex loss block with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      tx: LossBlock::new(),
      rx: LossBlock::new(),
    }
  }

  /// Sets the loss block for transmitted messages.
  ///
  /// The default is [`LossBlock::new()`].
  ///
  /// [`LossBlock::new()`]: LossBlock::new
  #[must_use]
  pub fn tx(mut self, tx: LossBlock) -> Self {
    self.tx = tx;
    self
  }

  /// Sets the loss block for received messages.
  ///
  /// The default is [`LossBlock::new()`].
  ///
  /// [`LossBlock::new()`]: LossBlock::new
  #[must_use]
  pub fn rx(mut self, rx: LossBlock) -> Self {
    self.rx = rx;
    self
  }

  /// Sets the same loss block for transmitted and received messages.
  ///
  /// The default is [`LossBlock::new()`] for both directions.
  ///
  /// [`LossBlock::new()`]: LossBlock::new
  #[must_use]
  pub fn both(mut self, block: LossBlock) -> Self {
    self.tx = block.clone();
    self.rx = block;
    self
  }
}

impl Default for DuplexLossBlock {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: 'static, R: 'static, E: Error + 'static>
  Attach<(Duplex<T, R, E>, Duplex<R, T, E>)> for DuplexLossBlock
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
  for DuplexLossBlock
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
