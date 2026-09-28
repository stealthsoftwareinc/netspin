//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::loss::LossBlock;
use crate::loss::LossModel;

#[derive(Clone)]
struct DropFirst {
  first: bool,
}

impl LossModel for DropFirst {
  fn should_drop(&mut self) -> bool {
    core::mem::replace(&mut self.first, false)
  }
}

// A configured loss model controls which messages are dropped, and
// cloning the block clones the model.
#[test]
fn test() {
  let block = LossBlock::new().model(DropFirst { first: true });
  for mut block in [block.clone(), block] {
    assert!(block.model.should_drop());
    assert!(!block.model.should_drop());
  }
}
