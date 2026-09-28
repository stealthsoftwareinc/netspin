//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::ReceiveOptions;
use crate::Task;

// Rx::try_receive_with_options() returns immediately regardless of the
// configured timeout.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (_tx, rx) = Channel::<()>::new().pair();
  rx.set_receive_timeout(Some(Duration::from_nanos(25)));
  let receiver = executor.spawn(async move {
    let message = rx
      .try_receive_with_options(
        ReceiveOptions::new().timeout(Duration::from_nanos(50)),
      )
      .unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(receiver.output(), Some((None, Moment::ZERO)));
}
