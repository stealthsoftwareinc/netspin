//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::test_error::TestError;

// A failure after a partial send reports the sent size and unsent tail.
#[test]
fn test() {
  let (tx, rx) = Channel::<Vec<u8>, TestError>::default()
    .capacity(Some(2))
    .message_size(Vec::len)
    .message_split(|message, size| Some(message.split_off(size)))
    .pair();
  let mut executor = Executor::new();
  let sender =
    executor.spawn(async move { tx.send(vec![1, 2, 3, 4]).await });
  executor.spawn(async move {
    rx.fail(TestError("failed"));
  });
  executor.run();

  let error = sender.output().unwrap().unwrap_err();
  assert!(matches!(
    error.cause,
    ChannelError::Failed(info) if *info == TestError("failed"),
  ));
  assert_eq!(error.sent, 2);
  assert_eq!(error.unsent, vec![3, 4]);
}
