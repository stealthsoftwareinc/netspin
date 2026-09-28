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

// An upstream abort follows messages delayed by a LatencyBlock.
#[test]
fn test() {
  let (input, target) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  let output = LatencyBlock::new()
    .model(FixedLatency::new(Duration::from_nanos(5)))
    .attach(&mut executor, target);
  let abort: ChannelErrorInfo = TestError("aborted").into();
  let reported_abort = abort.clone();
  executor.spawn(async move {
    input.send(7).await.unwrap();
    input.abort_with(reported_abort);
  });
  let receiver = executor.spawn(async move {
    let message = output.receive().await;
    let message_moment = Task::now();
    let abort = output.receive().await;
    let abort_moment = Task::now();
    (message, message_moment, abort, abort_moment)
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((
      Ok(7),
      Moment::from_nanos(5),
      Err(ChannelError::Aborted(abort)),
      Moment::from_nanos(5),
    )),
  );
}
