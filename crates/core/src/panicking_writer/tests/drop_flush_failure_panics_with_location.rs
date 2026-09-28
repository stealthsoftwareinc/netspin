//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::io;
use std::io::Write;

use crate::PanickingWriter;

struct Writer;

impl Write for Writer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Err(io::Error::other("test flush failure"))
  }
}

#[test]
#[should_panic(
  expected = "Failed to flush to example.txt: test flush failure"
)]
fn test() {
  let _writer = PanickingWriter::new(Writer, "example.txt");
}
