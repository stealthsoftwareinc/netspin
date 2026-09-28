//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::SendError;
use crate::test_error::TestError;

// A receiver failure interrupts a backpressured send.
#[test]
fn test() {
  let (tx, rx) = Channel::new().fallible::<TestError>().pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    tx.send(2).await
  });
  executor.spawn(async move {
    rx.fail(reported_failure);
  });
  executor.run();

  assert_eq!(
    sender.output(),
    Some(Err(SendError {
      cause: ChannelError::Failed(failure),
      sent: 0,
      unsent: 2,
    })),
  );
}
