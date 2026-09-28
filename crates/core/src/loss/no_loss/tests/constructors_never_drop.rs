//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::loss::LossModel;
use crate::loss::NoLoss;

// NoLoss constructors produce models that never drop messages.
#[expect(
  clippy::default_constructed_unit_structs,
  reason = "This test exercises the Default implementation."
)]
#[test]
fn test() {
  for mut model in [NoLoss::new(), NoLoss::default()] {
    assert!(!model.should_drop());
    assert!(!model.should_drop());
  }
}
