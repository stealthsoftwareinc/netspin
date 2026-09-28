//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;
use core::fmt;
use core::ops::Deref;
use std::rc::Rc;

/// Shared information about an error reported through a channel.
///
/// Cloning this type shares the error instead of requiring the error
/// itself to implement [`Clone`].
///
/// [`Clone`]: core::clone::Clone
pub struct ChannelErrorInfo<E: Error + ?Sized + 'static = dyn Error> {
  error: Rc<E>,
}

impl<E: Error + 'static> ChannelErrorInfo<E> {
  /// Creates channel error information containing `error`.
  #[must_use]
  pub fn new(error: E) -> Self {
    Self {
      error: Rc::new(error),
    }
  }
}

impl<E: Error + ?Sized + 'static> ChannelErrorInfo<E> {
  /// Returns the reported error.
  #[must_use]
  pub fn error(&self) -> &E {
    &self.error
  }
}

impl ChannelErrorInfo {
  /// Returns the reported error as `E`, if it has type `E`.
  #[must_use]
  pub fn downcast_ref<E: Error + 'static>(&self) -> Option<&E> {
    self.error.downcast_ref()
  }
}

impl<E: Error + ?Sized + 'static> Clone for ChannelErrorInfo<E> {
  fn clone(&self) -> Self {
    Self {
      error: self.error.clone(),
    }
  }
}

impl<E: Error + fmt::Debug + ?Sized + 'static> fmt::Debug
  for ChannelErrorInfo<E>
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.error.fmt(f)
  }
}

impl<E: Error + fmt::Display + ?Sized + 'static> fmt::Display
  for ChannelErrorInfo<E>
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.error.fmt(f)
  }
}

impl<E: Error + ?Sized + 'static> Deref for ChannelErrorInfo<E> {
  type Target = E;

  fn deref(&self) -> &Self::Target {
    self.error()
  }
}

impl<E: Error + Eq + 'static> Eq for ChannelErrorInfo<E> {}

impl<E: Error + PartialEq + 'static> PartialEq for ChannelErrorInfo<E> {
  fn eq(&self, other: &Self) -> bool {
    self.error == other.error
  }
}

impl<E: Error + 'static> From<E> for ChannelErrorInfo<E> {
  fn from(error: E) -> Self {
    Self::new(error)
  }
}

impl<E: Error + 'static> From<E> for ChannelErrorInfo {
  fn from(error: E) -> Self {
    Self {
      error: Rc::new(error),
    }
  }
}

impl<E: Error + 'static> From<ChannelErrorInfo<E>>
  for ChannelErrorInfo
{
  fn from(info: ChannelErrorInfo<E>) -> Self {
    Self { error: info.error }
  }
}

#[cfg(test)]
mod tests;
