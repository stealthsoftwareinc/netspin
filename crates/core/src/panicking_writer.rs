//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::io;
use std::io::IoSlice;
use std::io::Write;

/// A writer that panics when an I/O operation fails.
///
/// The writer is flushed when this value is dropped.
/// A failure during that flush also causes a panic.
pub struct PanickingWriter<W: Write> {
  location: String,
  writer: W,
}

impl<W: Write> PanickingWriter<W> {
  /// Wraps `writer`, adding `location` to any panic message.
  pub fn new(writer: W, location: impl Into<String>) -> Self {
    Self {
      location: location.into(),
      writer,
    }
  }

  /// Returns the location included in panic messages.
  #[must_use]
  pub fn location(&self) -> &str {
    &self.location
  }

  fn write_failed(&self, error: io::Error) -> ! {
    panic!("Failed to write to {}: {error}", self.location);
  }

  fn flush_failed(&self, error: io::Error) -> ! {
    panic!("Failed to flush to {}: {error}", self.location);
  }
}

impl<W: Write> Write for PanickingWriter<W> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self.writer.write(buf) {
      Ok(n) => Ok(n),
      Err(e) => self.write_failed(e),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self.writer.flush() {
      Ok(()) => Ok(()),
      Err(e) => self.flush_failed(e),
    }
  }

  fn write_vectored(
    &mut self,
    bufs: &[IoSlice<'_>],
  ) -> io::Result<usize> {
    match self.writer.write_vectored(bufs) {
      Ok(n) => Ok(n),
      Err(e) => self.write_failed(e),
    }
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    match self.writer.write_all(buf) {
      Ok(()) => Ok(()),
      Err(e) => self.write_failed(e),
    }
  }

  fn write_fmt(
    &mut self,
    args: core::fmt::Arguments<'_>,
  ) -> io::Result<()> {
    match self.writer.write_fmt(args) {
      Ok(()) => Ok(()),
      Err(e) => self.write_failed(e),
    }
  }
}

impl<W: Write> Drop for PanickingWriter<W> {
  fn drop(&mut self) {
    if let Err(e) = self.writer.flush() {
      self.flush_failed(e);
    }
  }
}

#[cfg(test)]
mod tests;
