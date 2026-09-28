//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::latency::DuplexLatencyBlock;

struct RxMessage;
struct TxMessage;

// DuplexLatencyBlock constructors and cloning support different,
// non-clone transmitted and received message types.
#[test]
fn test() {
  for block in [
    DuplexLatencyBlock::<TxMessage, RxMessage>::new(),
    DuplexLatencyBlock::<TxMessage, RxMessage>::default(),
  ] {
    let _: DuplexLatencyBlock<TxMessage, RxMessage> = block.clone();
  }
}
