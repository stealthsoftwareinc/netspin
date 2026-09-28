//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::test_error::TestError;

// Partially releasing a reserved message frees its channel space.
#[test]
fn test() {
  let completed = Rc::new(Cell::new(None));
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new()
    .capacity(Some(2))
    .message_size(Vec::len)
    .fallible::<TestError>()
    .pair();
  executor.spawn({
    let completed = completed.clone();
    async move {
      tx.send(vec![1, 2]).await.unwrap();
      tx.send(vec![3]).await.unwrap();
      completed.set(Some(Task::now()));
    }
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_secs(1)).await;
    let (message, mut reservation) =
      rx.expect_receive_reserved().unwrap();
    assert_eq!(message, vec![1, 2]);
    Task::sleep(Duration::from_secs(1)).await;
    reservation.release(1);
    Task::sleep(Duration::from_secs(1)).await;
  });
  executor.run();

  assert_eq!(
    completed.get(),
    Some(Moment::ZERO + Duration::from_secs(2)),
  );
}
