//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

/// A duplex endpoint that can be split into sending and receiving
/// endpoints.
pub trait Split {
  /// The sending endpoint type.
  type Tx;

  /// The receiving endpoint type.
  type Rx;

  /// Splits this endpoint into sending and receiving endpoints.
  fn split(self) -> (Self::Tx, Self::Rx);
}
