//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Rx;
use crate::Task;

struct Block;

impl Attach<()> for Block {
  type Surface = Rx<Moment>;

  fn attach(
    self,
    spawner: &mut impl crate::Spawn,
    _target: (),
  ) -> Self::Surface {
    let (tx, rx) = Channel::new().pair();
    spawner.spawn(async move {
      tx.send(Task::now()).await.unwrap();
    });
    rx
  }
}

// Task::spawner() spawns attachment tasks at the current task's moment.
#[test]
fn test() {
  let mut executor = Executor::default();
  let output = executor.spawn(async {
    Task::sleep(Duration::from_secs(1)).await;
    let rx = Block.attach(&mut Task::spawner(), ());
    rx.receive().await.unwrap()
  });
  executor.run();

  assert_eq!(
    output.output(),
    Some(Moment::ZERO + Duration::from_secs(1)),
  );
}
