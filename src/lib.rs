#![no_std]

use core::fmt::{self, Write};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

static DEFAULT_LOGGER: EtsPrintfLogger = EtsPrintfLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&DEFAULT_LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

struct EtsPrintfLogger;

impl log::Log for EtsPrintfLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            const SIZE: usize = 256;
            const SUFFIX: usize = 2;
            const ELLIPSES: usize = 3;
            let mut buffer = [0u8; SIZE];
            // Always leave enough space for `\n\0`
            let mut writer = Wrapper::new(&mut buffer[..SIZE - SUFFIX]);
            let res = write!(writer, "{}", record.args());
            let offset = writer.offset();
            if res.is_err() && offset >= ELLIPSES {
                // If couldn't write the whole string and there's space, replace the
                // end to indicate it's truncated.
                let ellipses: &[u8; ELLIPSES] = b"...";
                buffer[offset - ELLIPSES..offset].copy_from_slice(ellipses);
            }
            // Write newline and nul-terminating 0 (as expected by C API)
            let suffix: &[u8; SUFFIX] = b"\n\0";
            buffer[offset..offset + SUFFIX].copy_from_slice(suffix);
            unsafe {
                esp_idf_sys::ets_printf(buffer.as_ptr() as *const _);
            }
        }
    }

    fn flush(&self) {}
}

/// Based off:
/// https://stackoverflow.com/questions/39488327/how-to-format-output-to-a-byte-array-with-no-std-and-no-allocator
struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Wrapper { buf, offset: 0 }
    }
    fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a> fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Skip over already-copied data leaving space for '\0'
        let remainder = &mut self.buf[self.offset..];
        let bytes = s.as_bytes();
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() {
            return Err(fmt::Error);
        }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}
