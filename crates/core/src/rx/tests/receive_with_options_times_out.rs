//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::ReceiveOptions;
use crate::Task;

// Rx::receive_with_options() returns an error when its configured
// timeout expires.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (_tx, rx) = Channel::<()>::new().pair();
  let receiver = executor.spawn(async move {
    let error = rx
      .receive_with_options(
        ReceiveOptions::new().timeout(Duration::from_nanos(50)),
      )
      .await
      .unwrap_err();
    (error, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((ChannelError::TimedOut, Moment::from_nanos(50))),
  );
}
