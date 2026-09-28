//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll_with_timeout;

// A timeout with no ready messages returns None at the deadline.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (_tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let receiver = executor.spawn(async move {
    let ready =
      poll_with_timeout(&[&rx], Duration::from_nanos(50)).await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (None, Moment::from_nanos(50))
  );
}
