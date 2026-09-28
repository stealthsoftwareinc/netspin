//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::Task;

// Rx::receive() uses the endpoint's configured receiving timeout.
#[test]
fn test() {
  let (_tx, rx) = Channel::<()>::new().pair();
  rx.set_receive_timeout(Some(Duration::from_nanos(7)));
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    let result = rx.receive().await;
    (result, Task::now())
  });
  executor.run();

  assert_eq!(
    receiver.output(),
    Some((Err(ChannelError::TimedOut), Moment::from_nanos(7))),
  );
}
