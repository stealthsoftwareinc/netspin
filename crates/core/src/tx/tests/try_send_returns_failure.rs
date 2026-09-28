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

// Tx::try_send() returns the channel's sticky failure.
#[test]
fn test() {
  let (tx, rx) = Channel::new().fallible::<TestError>().pair();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  rx.fail(failure.clone());
  let mut executor = Executor::new();
  let sender =
    executor.spawn(async move { (tx.try_send(1), tx.try_send(2)) });
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
        cause: ChannelError::Failed(failure),
        sent: 0,
        unsent: 2,
      }),
    )),
  );
}
