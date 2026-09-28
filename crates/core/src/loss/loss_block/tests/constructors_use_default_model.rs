//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::loss::LossBlock;

// LossBlock constructors use a model that never drops messages.
#[test]
fn test() {
  for mut block in [LossBlock::new(), LossBlock::default()] {
    assert!(!block.model.should_drop());
    assert!(!block.model.should_drop());
  }
}
