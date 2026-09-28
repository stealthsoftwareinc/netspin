//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// Tx::try_send_with_options() returns immediately regardless of the
// configured timeout.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::new().pair();
  tx.set_send_timeout(Some(Duration::from_nanos(25)));
  let sender = executor.spawn(async move {
    assert_eq!(tx.try_send(1).unwrap(), None);
    let message = tx
      .try_send_with_options(
        2,
        SendOptions::new().timeout(Duration::from_nanos(50)),
      )
      .unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(sender.output(), Some((Some(2), Moment::ZERO)));
}
