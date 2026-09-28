//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::Pump;
use crate::test_error::TestError;

// An upstream abort follows preceding messages through a pump.
#[test]
fn test() {
  let (input, pump_input) = Channel::<u8, TestError>::default().pair();
  let (pump_output, output) =
    Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  Pump::new().attach(&mut executor, (pump_input, pump_output));
  let abort: ChannelErrorInfo = TestError("aborted").into();
  let reported_abort = abort.clone();
  executor.spawn(async move {
    input.send(1).await.unwrap();
    input.abort_with(reported_abort);
  });
  let receiver = executor.spawn(async move {
    (output.receive().await, output.receive().await)
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((Ok(1), Err(ChannelError::Aborted(abort)))),
  );
}
