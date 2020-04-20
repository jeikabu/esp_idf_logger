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
            const BUFFER_SIZE: usize = 128;
            const SUFFIX_SIZE: usize = 2;
            const ELLIPSES_SIZE: usize = 3;
            // C API expects log output ends with newline and nul-terminating 0
            let buffer_suffix: &[u8; SUFFIX_SIZE] = b"\n\0";
            let mut string_buffer = [0u8; BUFFER_SIZE];
            // Always leave enough space for `buffer_suffix`
            let mut writer = Wrapper::new(&mut string_buffer[..BUFFER_SIZE - SUFFIX_SIZE]);
            let res = write!(writer, "{}", record.args());
            let mut offset = writer.offset();
            if res.is_err() {
                if offset >= ELLIPSES_SIZE {
                    // If couldn't write the whole string and there's space, replace the
                    // end to indicate it's truncated.
                    let ellipses: &[u8; ELLIPSES_SIZE] = b"...";
                    string_buffer[offset - ELLIPSES_SIZE..offset].copy_from_slice(ellipses);
                } else {
                    let error_msg = b"logger OOPS";
                    offset = error_msg.len();
                    string_buffer[..offset].copy_from_slice(error_msg);
                }
            }
            
            string_buffer[offset..offset + SUFFIX_SIZE].copy_from_slice(buffer_suffix);
            unsafe {
                esp_idf_sys::ets_printf(string_buffer.as_ptr() as *const _);
            }
        }
    }

    fn flush(&self) {
        // TODO: not sure if this should be `esp_idf_sys::fflush()` or `esp_idf_sys::uart_flush()`
    }
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
        // Skip over already-copied data
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
