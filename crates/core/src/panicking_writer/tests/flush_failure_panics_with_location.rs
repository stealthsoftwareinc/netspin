//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::io;
use std::io::Write;

use crate::PanickingWriter;

struct Writer {
  fail: bool,
}

impl Write for Writer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    if self.fail {
      self.fail = false;
      Err(io::Error::other("test flush failure"))
    } else {
      Ok(())
    }
  }
}

#[test]
#[should_panic(
  expected = "Failed to flush to example.txt: test flush failure"
)]
fn test() {
  let mut writer =
    PanickingWriter::new(Writer { fail: true }, "example.txt");
  writer.flush().unwrap();
}
