//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;
use core::fmt;

use crate::ChannelError;

/// An error sending a message through a channel.
#[derive(Eq, PartialEq)]
#[non_exhaustive]
pub struct SendError<T, E: Error + 'static> {
  /// The cause of the error.
  pub cause: ChannelError<E>,

  /// The total message size accepted by the channel before the error
  /// occurred.
  ///
  /// This is measured using the channel's message-size function.
  /// It does not indicate that the receiver observed the sent data.
  pub sent: usize,

  /// The part of the message that remains unsent.
  pub unsent: T,
}

impl<T, E: Error + 'static> fmt::Debug for SendError<T, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SendError")
      .field("cause", &self.cause)
      .field("sent", &self.sent)
      .field("unsent", &format_args!("..."))
      .finish()
  }
}

impl<T, E: Error + 'static> fmt::Display for SendError<T, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.cause.fmt(f)
  }
}

impl<T, E: Error + 'static> Error for SendError<T, E> {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(&self.cause)
  }
}
