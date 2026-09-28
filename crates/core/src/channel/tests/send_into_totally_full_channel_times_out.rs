//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::ChannelOrder;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// A send into a totally full channel returns a timeout error with the
// unsent message when space does not become available.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::<i64>::new()
    .capacity(Some(1))
    .local(false)
    .order(ChannelOrder::Unordered)
    .pair();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    let options = SendOptions::new().timeout(Duration::from_nanos(50));
    let error = tx.send_with_options(2, options).await.unwrap_err();
    (error, Task::now())
  });
  executor.run();
  let (error, moment) = sender.output().unwrap();
  assert_eq!(error.cause, ChannelError::TimedOut);
  assert_eq!(error.sent, 0);
  assert_eq!(error.unsent, 2);
  assert_eq!(moment, Moment::from_nanos(50));
}
