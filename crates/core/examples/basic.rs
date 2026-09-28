//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use netspin_core::Channel;
use netspin_core::Executor;
use netspin_core::Task;

fn main() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().pair();

  executor.spawn(async move {
    for message in ["Hello", "from", "Alice"] {
      Task::sleep(Duration::from_secs(1)).await;
      tx.send(message).await.unwrap();
    }
  });

  executor.spawn(async move {
    while let Ok(message) = rx.receive().await {
      println!("t={}: {message}", Task::now());
    }
  });

  executor.run();
}
