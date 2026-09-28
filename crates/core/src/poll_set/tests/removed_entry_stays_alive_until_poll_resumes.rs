//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::Task;

// Removing an entry from a suspended poll defers closing its channel
// until the poll resumes and clears its original registrations.
#[test]
fn test() {
  for timeout in [None, Some(Duration::from_nanos(50))] {
    let mut executor = Executor::new();
    let poll_set = PollSet::new();
    let (tx1, rx1) = Channel::<()>::new().pair();
    let (tx2, rx2) = Channel::<()>::new().pair();
    let rx1 = poll_set.insert(rx1);
    let rx2 = poll_set.insert(rx2);
    let key2 = rx2.key();
    let waiter = executor.spawn(async move {
      let ready = match timeout {
        Some(timeout) => poll_set.poll_with_timeout(timeout).await,
        None => Some(poll_set.poll().await),
      };
      assert_eq!(ready, Some(key2));
      rx2.expect_receive().unwrap();
      Task::now()
    });
    let mutator = executor.spawn(async move {
      Task::sleep(Duration::from_nanos(10)).await;
      drop(rx1);
      assert!(tx1.error().is_none());
      Task::sleep(Duration::from_nanos(10)).await;
      assert!(tx1.error().is_some());
      tx2.send(()).await.unwrap();
    });
    executor.run();
    assert_eq!(waiter.output(), Some(Moment::from_nanos(20)));
    assert!(mutator.output().is_some());
  }
}
