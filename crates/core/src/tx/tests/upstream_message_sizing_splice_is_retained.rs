//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing retains the upstream channel's message sizing and
// splitting behavior.
#[test]
fn test() {
  let (upstream_tx, upstream_rx) = Channel::new()
    .capacity(Some(4))
    .message_size(Vec::len)
    .message_split(|message: &mut Vec<_>, limit| {
      (limit < message.len()).then(|| message.split_off(limit))
    })
    .pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
  let mut executor = Executor::new();
  let result = executor.spawn(async move {
    let unsent =
      upstream_tx.try_send(vec![1, 2, 3, 4, 5]).unwrap().unwrap();
    let message = downstream_rx.receive().await.unwrap();
    (unsent, message)
  });
  executor.run();
  assert_eq!(result.output(), Some((vec![5], vec![1, 2, 3, 4])));
}
