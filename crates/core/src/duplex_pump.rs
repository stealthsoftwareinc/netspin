//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Duplex;
use crate::Pump;
use crate::Spawn;

/// Pumps messages between two duplex endpoints in both directions.
#[derive(Default)]
pub struct DuplexPump;

impl DuplexPump {
  /// Creates a duplex pump.
  #[must_use]
  pub fn new() -> Self {
    Self
  }
}

impl<T: 'static, R: 'static, E: Error + 'static>
  Attach<(Duplex<T, R, E>, Duplex<R, T, E>)> for DuplexPump
{
  type Surface = ();

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: (Duplex<T, R, E>, Duplex<R, T, E>),
  ) -> Self::Surface {
    let (a, b) = target;
    Pump::new().attach(spawner, (a.rx, b.tx));
    Pump::new().attach(spawner, (b.rx, a.tx));
  }
}

impl<T: 'static, R: 'static, E: Error + 'static> Attach<Duplex<T, R, E>>
  for DuplexPump
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
