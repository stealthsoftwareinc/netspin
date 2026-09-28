//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// A parked receiver's delayed notification respects the horizon.
// Both completed and abandoned futures release their captured data,
// and abandoning registrations leaves no running task behind.
#[test]
fn test() {
  let mut executor = Executor::new().horizon(Duration::from_nanos(10));
  let captured = Rc::new(());
  let mut outputs = Vec::new();
  for arrival in 9..=11 {
    let (tx, rx) = Channel::<u64>::new().local(false).pair();
    outputs.push(executor.spawn({
      let captured = captured.clone();
      async move {
        let message = rx.receive().await.unwrap();
        drop(captured);
        (message, Task::now())
      }
    }));
    executor.spawn(async move {
      Task::sleep(Duration::from_nanos(5)).await;
      tx.send_with_options(
        arrival,
        SendOptions::new().latency(Duration::from_nanos(arrival - 5)),
      )
      .await
      .unwrap();
      Task::sleep(Duration::from_nanos(20)).await;
    });
  }
  executor.run();
  let results: Vec<_> =
    outputs.into_iter().map(|x| x.output()).collect();
  assert_eq!(
    results,
    vec![Some((9, Moment::from_nanos(9))), None, None]
  );
  assert_eq!(Rc::strong_count(&captured), 1);
  assert_eq!(Task::try_now(), None);
  assert!(!Task::is_running());
}
