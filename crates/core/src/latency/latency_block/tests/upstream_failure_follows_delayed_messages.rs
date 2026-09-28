//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;
use crate::test_error::TestError;

// An upstream failure follows messages delayed by a LatencyBlock.
#[test]
fn test() {
  let (input, target) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  let output = LatencyBlock::new()
    .model(FixedLatency::new(Duration::from_nanos(5)))
    .attach(&mut executor, target);
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  executor.spawn(async move {
    input.send(7).await.unwrap();
    input.fail(reported_failure);
  });
  let receiver = executor.spawn(async move {
    let message = output.receive().await;
    let message_moment = Task::now();
    let failure = output.receive().await;
    let failure_moment = Task::now();
    (message, message_moment, failure, failure_moment)
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((
      Ok(7),
      Moment::from_nanos(5),
      Err(ChannelError::Failed(failure)),
      Moment::from_nanos(5),
    )),
  );
}
