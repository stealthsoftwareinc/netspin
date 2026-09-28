//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::fmt;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TestError(pub(crate) &'static str);

impl fmt::Display for TestError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.0)
  }
}

impl core::error::Error for TestError {}
