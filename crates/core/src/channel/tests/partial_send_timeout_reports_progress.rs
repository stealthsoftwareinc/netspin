//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::SendOptions;

// A timeout after a partial send reports the sent size and unsent tail.
#[test]
fn test() {
  let (tx, _rx) = Channel::<Vec<u8>>::default()
    .capacity(Some(2))
    .message_size(Vec::len)
    .message_split(|message, size| Some(message.split_off(size)))
    .pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.send_with_options(
      vec![1, 2, 3, 4],
      SendOptions::new().timeout(Duration::from_nanos(5)),
    )
    .await
  });
  executor.run();

  let error = sender.output().unwrap().unwrap_err();
  assert_eq!(error.cause, ChannelError::TimedOut);
  assert_eq!(error.sent, 2);
  assert_eq!(error.unsent, vec![3, 4]);
}
