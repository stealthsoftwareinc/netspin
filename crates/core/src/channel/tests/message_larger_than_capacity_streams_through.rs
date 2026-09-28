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

// Sending a message that's larger than the channel capacity streams it
// through in pieces when the channel has a message_split function.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<Vec<u8>>::new()
    .capacity(Some(4))
    .local(false)
    .message_size(|m: &Vec<u8>| m.len())
    .message_split(|m, size| Some(m.split_off(size)))
    .pair();
  let sender = executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx.send_with_options(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], options)
      .await
      .unwrap();
  });
  let receiver = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..3 {
      let message = rx.receive().await.unwrap();
      log.push((message, Task::now()));
    }
    log
  });
  executor.run();
  assert_eq!(sender.output(), Some(()));
  assert_eq!(
    receiver.output().unwrap(),
    [
      (vec![0, 1, 2, 3], Moment::from_nanos(100)),
      (vec![4, 5, 6, 7], Moment::from_nanos(200)),
      (vec![8, 9], Moment::from_nanos(300))
    ]
  );
}
