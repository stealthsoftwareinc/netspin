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

// The first terminal event is returned by every subsequent operation.
#[test]
fn test() {
  let (tx, rx) = Channel::<u8, TestError>::default().pair();
  let failure = ChannelErrorInfo::new(TestError("first"));
  tx.fail(failure.clone());
  rx.fail(TestError("second"));
  let mut executor = Executor::new();
  let sender =
    executor.spawn(async move { (tx.send(1).await, tx.send(2).await) });
  let receiver = executor
    .spawn(async move { (rx.receive().await, rx.receive().await) });
  executor.run();
  assert_eq!(
    sender.output(),
    Some((
      Err(SendError {
        cause: ChannelError::Failed(failure.clone()),
        sent: 0,
        unsent: 1,
      }),
      Err(SendError {
        cause: ChannelError::Failed(failure.clone()),
        sent: 0,
        unsent: 2,
      }),
    )),
  );
  let expected_receive = Err(ChannelError::Failed(failure));
  assert_eq!(
    receiver.output(),
    Some((expected_receive.clone(), expected_receive)),
  );
}
