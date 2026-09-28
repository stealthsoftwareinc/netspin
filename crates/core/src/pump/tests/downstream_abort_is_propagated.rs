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
use crate::SendError;
use crate::test_error::TestError;

// A downstream abort interrupts and aborts an upstream sender.
#[test]
fn test() {
  let (input, pump_input) = Channel::<u8, TestError>::default().pair();
  let (pump_output, output) =
    Channel::<u8, TestError>::default().pair();
  let abort: ChannelErrorInfo = TestError("aborted").into();
  output.abort_with(abort.clone());
  let mut executor = Executor::new();
  Pump::new().attach(&mut executor, (pump_input, pump_output));
  let sender = executor.spawn(async move {
    let first = input.send(1).await;
    let second = input.send(2).await;
    (first, second)
  });
  executor.run();
  assert_eq!(
    sender.output(),
    Some((
      Ok(()),
      Err(SendError {
        cause: ChannelError::Aborted(abort),
        sent: 0,
        unsent: 2,
      }),
    )),
  );
}
