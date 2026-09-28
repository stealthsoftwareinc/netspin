//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::io;
use std::io::Write;

use crate::PanickingWriter;

struct Writer;

impl Write for Writer {
  fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
    Err(io::Error::other("test write failure"))
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

#[test]
#[should_panic(
  expected = "Failed to write to example.txt: test write failure"
)]
fn test() {
  let mut writer = PanickingWriter::new(Writer, "example.txt");
  writer.write_all(b"hello").unwrap();
}
