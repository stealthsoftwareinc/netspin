//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::fmt;

use crate::ChannelErrorInfo;

#[derive(Debug, Eq, PartialEq)]
struct TestError(Box<u8>);

impl fmt::Display for TestError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("test error")
  }
}

impl core::error::Error for TestError {}

// Erased channel error information can be cloned without cloning its
// contained error and preserves the concrete error type.
#[test]
fn test() {
  let info: ChannelErrorInfo = TestError(Box::new(7)).into();
  let clone = info.clone();

  assert_eq!(*info.downcast_ref::<TestError>().unwrap().0, 7);
  assert_eq!(*clone.downcast_ref::<TestError>().unwrap().0, 7);
}
