//! Built-in functions - minimal implementation of core Python builtins
//!
//! Design: Only essential functions, optimized for compiled code.

/// print() - minimal implementation
#[no_mangle]
pub extern "C" fn typthon_print_int(val: i64) {
    // TODO: Implement without std
    // For now, assume std for prototyping
}

#[no_mangle]
pub extern "C" fn typthon_print_str(ptr: *const u8, len: usize) {
    // TODO: Implement
}

/// len() - length of collections
#[no_mangle]
pub extern "C" fn typthon_len(obj: *const u8) -> usize {
    // TODO: Read length from object header
    0
}

/// range() - iterator support
#[no_mangle]
pub extern "C" fn typthon_range(start: i64, end: i64, step: i64) -> Range {
    Range { current: start, end, step }
}

#[repr(C)]
pub struct Range {
    current: i64,
    end: i64,
    step: i64,
}

impl Range {
    #[no_mangle]
    pub extern "C" fn next(&mut self) -> Option<i64> {
        if self.current < self.end {
            let val = self.current;
            self.current += self.step;
            Some(val)
        } else {
            None
        }
    }
}

