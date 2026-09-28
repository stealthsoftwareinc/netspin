//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::PollTarget;

/// An object that can be polled for readiness.
///
/// Constructing a [`PollTarget`] directly is restricted to this crate.
/// Implementations outside this crate must delegate to an existing
/// [`Pollable`] object by returning its [`poll_target()`]:
///
/// ```
/// # use netspin_core::PollTarget;
/// # use netspin_core::Pollable;
/// # use netspin_core::Rx;
/// struct Foo { rx: Rx<()> }
/// // Polling a Foo is the same as polling its internal rx.
/// impl Pollable for Foo {
///   fn poll_target(&self) -> PollTarget {
///     return self.rx.poll_target();
///   }
/// }
/// ```
///
/// While a [`Pollable`] is in the middle of a poll operation, every
/// call to [`poll_target()`] must return the same target.
/// Implementations will usually return the same target throughout the
/// entire lifetime of the [`Pollable`], but switching the target is
/// permitted as long as it's done between poll operations.
///
/// [`PollTarget`]: crate::PollTarget
/// [`Pollable`]: crate::Pollable
/// [`poll_target()`]: crate::Pollable::poll_target()
pub trait Pollable {
  /// Returns the object's polling target.
  fn poll_target(&self) -> PollTarget<'_>;
}

impl<P: Pollable + ?Sized> Pollable for &P {
  fn poll_target(&self) -> PollTarget<'_> {
    P::poll_target(self)
  }
}
