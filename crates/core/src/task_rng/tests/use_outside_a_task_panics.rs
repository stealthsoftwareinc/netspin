//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use rand::Rng;

use crate::TaskRng;

// A TaskRng can only draw while an executor is running, so using one
// outside a task panics.
#[test]
#[should_panic(
  expected = "TaskRng::try_fill_bytes() must only be called within a task"
)]
fn test() {
  TaskRng.fill_bytes(&mut [0u8; 4]);
}
