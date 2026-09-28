//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::Channel;
use crate::ChannelOrder;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// A receiver looping on receive() takes messages in arrival order,
// with ties in send order, receiving each message at the later of
// its arrival and the previous receive moment. The expected log is
// computed from a seeded pseudorandom send schedule without using
// the channel.
#[test]
fn test() {
  let mut rng = StdRng::seed_from_u64(0);
  let mut plan = Vec::new();
  for _ in 0..100 {
    let pause = Duration::from_nanos(rng.random_range(0..10));
    let latency = Duration::from_nanos(rng.random_range(0..100));
    plan.push((pause, latency));
  }
  let mut arrivals = Vec::new();
  let mut clock = Moment::ZERO;
  for (i, &(pause, latency)) in plan.iter().enumerate() {
    clock += pause;
    arrivals.push((clock + latency, i as i64));
  }
  arrivals.sort();
  let mut expected = Vec::new();
  let mut clock = Moment::ZERO;
  for &(arrival, message) in &arrivals {
    clock = clock.max(arrival);
    expected.push((message, clock));
  }
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<i64>::new()
    .capacity(None)
    .local(false)
    .order(ChannelOrder::Unordered)
    .pair();
  executor.spawn(async move {
    for (i, (pause, latency)) in plan.into_iter().enumerate() {
      Task::sleep(pause).await;
      let options = SendOptions::new().latency(latency);
      tx.send_with_options(i as i64, options).await.unwrap();
    }
  });
  let receiver = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..100 {
      let message = rx.receive().await.unwrap();
      log.push((message, Task::now()));
    }
    log
  });
  executor.run();
  assert_eq!(receiver.output().unwrap(), expected);
}
