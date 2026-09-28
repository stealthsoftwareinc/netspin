//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Pump;
use crate::Rx;
use crate::Spawn;
use crate::Tx;
use crate::loss::LossModel;
use crate::loss::NoLoss;

/// Configures a loss block.
#[derive(Clone)]
pub struct LossBlock {
  model: Box<dyn LossModel>,
}

impl LossBlock {
  /// Creates a loss block with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      model: Box::new(NoLoss::new()),
    }
  }

  /// Sets the loss model.
  ///
  /// The default is [`NoLoss`].
  ///
  /// [`NoLoss`]: crate::loss::NoLoss
  #[must_use]
  pub fn model(mut self, model: impl LossModel + 'static) -> Self {
    self.model = Box::new(model);
    self
  }
}

impl Default for LossBlock {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: 'static, E: Error + 'static> Attach<Rx<T, E>> for LossBlock {
  type Surface = Rx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Rx<T, E>,
  ) -> Self::Surface {
    let Self { mut model } = self;
    let (output_tx, output_rx) = Channel::new().fallible().pair();
    spawner.spawn(async move {
      loop {
        let message = match target.receive().await {
          Ok(message) => message,
          Err(ChannelError::Aborted(info)) => {
            output_tx.abort_with(info);
            return;
          }
          Err(ChannelError::Closed) => return,
          Err(ChannelError::Failed(error)) => {
            output_tx.fail(error);
            return;
          }
          Err(ChannelError::TimedOut) => return,
        };
        if model.should_drop() {
          continue;
        }
        match output_tx.send(message).await {
          Ok(()) => {}
          Err(error) => match error.cause {
            ChannelError::Aborted(info) => {
              target.abort_with(info);
              return;
            }
            ChannelError::Closed => return,
            ChannelError::Failed(error) => {
              target.fail(error);
              return;
            }
            ChannelError::TimedOut => return,
          },
        }
      }
    });
    output_rx
  }
}

impl<T: 'static, E: Error + 'static> Attach<(Rx<T, E>, Tx<T, E>)>
  for LossBlock
{
  type Surface = ();

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: (Rx<T, E>, Tx<T, E>),
  ) -> Self::Surface {
    let (input, output) = target;
    let filtered = Self::attach(self, spawner, input);
    Pump::new().attach(spawner, (filtered, output));
  }
}

impl<T: 'static, E: Error + 'static> Attach<Tx<T, E>> for LossBlock {
  type Surface = Tx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Tx<T, E>,
  ) -> Self::Surface {
    let (a, b) = Channel::new().fallible().pair();
    Self::attach(self, spawner, (b, target));
    a
  }
}

#[cfg(test)]
mod tests;
