//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Rx;
use crate::Spawn;
use crate::Tx;

/// Pumps messages from a channel receiver to a channel transmitter.
#[derive(Default)]
pub struct Pump;

impl Pump {
  /// Creates a pump.
  #[must_use]
  pub fn new() -> Self {
    Self
  }
}

impl<T: 'static, E: Error + 'static> Attach<(Rx<T, E>, Tx<T, E>)>
  for Pump
{
  type Surface = ();

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: (Rx<T, E>, Tx<T, E>),
  ) -> Self::Surface {
    let (rx, tx) = target;
    spawner.spawn(async move {
      loop {
        match rx.receive().await {
          Ok(message) => match tx.send(message).await {
            Ok(()) => {}
            Err(error) => match error.cause {
              ChannelError::Aborted(info) => {
                rx.abort_with(info);
                return;
              }
              ChannelError::Closed => return,
              ChannelError::Failed(error) => {
                rx.fail(error);
                return;
              }
              ChannelError::TimedOut => return,
            },
          },
          Err(ChannelError::Aborted(info)) => {
            tx.abort_with(info);
            return;
          }
          Err(ChannelError::Closed) => return,
          Err(ChannelError::Failed(error)) => {
            tx.fail(error);
            return;
          }
          Err(ChannelError::TimedOut) => return,
        }
      }
    });
  }
}

impl<T: 'static, E: Error + 'static> Attach<Rx<T, E>> for Pump {
  type Surface = Rx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Rx<T, E>,
  ) -> Self::Surface {
    let (tx, rx) = Channel::new().fallible().pair();
    <Self as Attach<(Rx<T, E>, Tx<T, E>)>>::attach(
      self,
      spawner,
      (target, tx),
    );
    rx
  }
}

impl<T: 'static, E: Error + 'static> Attach<Tx<T, E>> for Pump {
  type Surface = Tx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Tx<T, E>,
  ) -> Self::Surface {
    let (tx, rx) = Channel::new().fallible().pair();
    <Self as Attach<(Rx<T, E>, Tx<T, E>)>>::attach(
      self,
      spawner,
      (rx, target),
    );
    tx
  }
}

#[cfg(test)]
mod tests;
