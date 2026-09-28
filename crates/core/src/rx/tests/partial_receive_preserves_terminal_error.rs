//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::test_error::TestError;

// A transmitter failure remains behind both pieces of a partially
// received message.
#[test]
fn test() {
  let (tx, rx) = Channel::new()
    .capacity(Some(4))
    .message_size(Vec::len)
    .message_split(|bytes: &mut Vec<u8>, size| {
      Some(bytes.split_off(size))
    })
    .fallible::<TestError>()
    .pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let result = executor.spawn(async move {
    tx.send(vec![1, 2, 3, 4]).await.unwrap();
    tx.fail(reported_failure);
    (
      rx.try_receive_up_to(2),
      rx.try_receive_up_to(2),
      rx.try_receive_up_to(2),
    )
  });
  executor.run();

  assert_eq!(
    result.output(),
    Some((
      Ok(Some(vec![1, 2])),
      Ok(Some(vec![3, 4])),
      Err(ChannelError::Failed(failure)),
    )),
  );
}
