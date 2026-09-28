//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;
use core::future::poll_fn;
use core::pin::pin;
use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// A parked receive resumes at the arrival time, without polling once
// at the earlier send time merely to schedule another suspension.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().local(false).pair();
  let receiver = executor.spawn(async move {
    let mut moments = Vec::new();
    let mut receive = pin!(rx.receive());
    let message = poll_fn(|cx| {
      moments.push(Task::now());
      receive.as_mut().poll(cx)
    })
    .await
    .unwrap();
    (message, moments)
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(5)).await;
    tx.send_with_options(
      7,
      SendOptions::new().latency(Duration::from_nanos(25)),
    )
    .await
    .unwrap();
  });
  executor.run();

  let result = receiver.output();
  assert_eq!(
    result,
    Some((7, vec![Moment::ZERO, Moment::from_nanos(30)])),
  );
}
