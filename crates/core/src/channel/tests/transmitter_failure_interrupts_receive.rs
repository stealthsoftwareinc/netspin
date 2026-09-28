//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::test_error::TestError;

// A transmitter failure interrupts a pending receive.
#[test]
fn test() {
  let (tx, rx) = Channel::<u8>::new().fallible::<TestError>().pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let receiver = executor.spawn(async move { rx.receive().await });
  executor.spawn(async move {
    tx.fail(reported_failure);
  });
  executor.run();

  assert_eq!(
    receiver.output(),
    Some(Err(ChannelError::Failed(failure))),
  );
}
