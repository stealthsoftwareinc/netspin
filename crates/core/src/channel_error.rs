//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;
use core::fmt;

use crate::ChannelErrorInfo;

/// An error from a channel operation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ChannelError<E: Error + 'static> {
  /// The channel aborted because of an error outside its declared
  /// failure type.
  Aborted(ChannelErrorInfo),

  /// The peer endpoint was dropped.
  Closed,

  /// An endpoint reported a failure.
  Failed(ChannelErrorInfo<E>),

  /// The operation timed out.
  ///
  /// This error is not terminal and does not change the channel state.
  TimedOut,
}

impl<E: Error + 'static> Clone for ChannelError<E> {
  fn clone(&self) -> Self {
    match self {
      Self::Aborted(info) => Self::Aborted(info.clone()),
      Self::Closed => Self::Closed,
      Self::Failed(info) => Self::Failed(info.clone()),
      Self::TimedOut => Self::TimedOut,
    }
  }
}

impl<E: Error + Eq + 'static> Eq for ChannelError<E> {}

impl<E: Error + PartialEq + 'static> PartialEq for ChannelError<E> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Aborted(_), Self::Aborted(_)) => true,
      (Self::Closed, Self::Closed) => true,
      (Self::Failed(a), Self::Failed(b)) => a == b,
      (Self::TimedOut, Self::TimedOut) => true,
      _ => false,
    }
  }
}

impl<E: Error + 'static> fmt::Display for ChannelError<E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Aborted(info) => write!(f, "channel aborted: {info}"),
      Self::Closed => f.write_str("channel closed"),
      Self::Failed(info) => info.fmt(f),
      Self::TimedOut => f.write_str("channel operation timed out"),
    }
  }
}

impl<E: Error + 'static> Error for ChannelError<E> {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Aborted(info) => Some(info.error()),
      Self::Closed => None,
      Self::Failed(info) => Some(info.error()),
      Self::TimedOut => None,
    }
  }
}

#[derive(Debug)]
pub(crate) struct UnspecifiedChannelAbort;

impl fmt::Display for UnspecifiedChannelAbort {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("no cause was reported")
  }
}

impl Error for UnspecifiedChannelAbort {}
