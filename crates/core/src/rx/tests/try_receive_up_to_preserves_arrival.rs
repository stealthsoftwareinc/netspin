//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;
use crate::poll;

// The retained suffix of a partially received message remains ready at
// the original arrival moment.
#[test]
fn test() {
  let (tx, rx) = Channel::new()
    .capacity(Some(8))
    .message_size(Vec::len)
    .message_split(|bytes: &mut Vec<u8>, size| {
      Some(bytes.split_off(size))
    })
    .local(false)
    .pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send_with_options(
      vec![1, 2, 3, 4],
      SendOptions::new().latency(Duration::from_nanos(7)),
    )
    .await
    .unwrap();
  });
  let result = executor.spawn(async move {
    assert_eq!(poll(&[&rx]).await, 0);
    let prefix = rx.try_receive_up_to(2).unwrap();
    let suffix = rx.try_receive_up_to(2).unwrap();
    let moment = Task::now();
    (prefix, suffix, moment)
  });
  executor.run();

  assert_eq!(
    result.output(),
    Some((Some(vec![1, 2]), Some(vec![3, 4]), Moment::from_nanos(7),)),
  );
}
