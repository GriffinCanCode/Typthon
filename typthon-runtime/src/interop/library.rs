//! Dynamic library loading and symbol resolution
//!
//! Platform-agnostic wrapper around dlopen/LoadLibrary.

use core::ffi::c_void;
use core::ptr::NonNull;

#[cfg(unix)]
use std::ffi::CString;

/// Handle to dynamically loaded library
pub struct Library {
    #[cfg(unix)]
    handle: NonNull<c_void>,
    #[cfg(windows)]
    handle: NonNull<c_void>,
}

impl Library {
    /// Load library by name
    ///
    /// Searches standard library paths. Use `load_path` for absolute paths.
    pub fn load(name: &str) -> Result<Self, LoadError> {
        Self::load_impl(name, false)
    }

    /// Load library from absolute path
    pub fn load_path(path: &str) -> Result<Self, LoadError> {
        Self::load_impl(path, true)
    }

    #[cfg(unix)]
    fn load_impl(name: &str, _is_path: bool) -> Result<Self, LoadError> {
        use std::os::raw::c_char;

        extern "C" {
            fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
            fn dlerror() -> *const c_char;
        }

        const RTLD_NOW: i32 = 2;

        let cname = CString::new(name).map_err(|_| LoadError::InvalidName)?;

        unsafe {
            let handle = dlopen(cname.as_ptr(), RTLD_NOW);
            NonNull::new(handle)
                .map(|h| Self { handle: h })
                .ok_or_else(|| {
                    let err = dlerror();
                    let msg = if !err.is_null() {
                        std::ffi::CStr::from_ptr(err)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        "Unknown error".into()
                    };
                    LoadError::LoadFailed(msg)
                })
        }
    }

    #[cfg(windows)]
    fn load_impl(name: &str, _is_path: bool) -> Result<Self, LoadError> {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;

        extern "system" {
            fn LoadLibraryW(filename: *const u16) -> *mut c_void;
            fn GetLastError() -> u32;
        }

        let wide: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(Some(0))
            .collect();

        unsafe {
            let handle = LoadLibraryW(wide.as_ptr());
            NonNull::new(handle)
                .map(|h| Self { handle: h })
                .ok_or_else(|| {
                    let code = GetLastError();
                    LoadError::LoadFailed(format!("Error code: {}", code))
                })
        }
    }

    /// Get function pointer by symbol name
    pub fn symbol(&self, name: &str) -> Result<*const (), SymbolError> {
        self.symbol_impl(name)
    }

    #[cfg(unix)]
    fn symbol_impl(&self, name: &str) -> Result<*const (), SymbolError> {
        use std::os::raw::c_char;

        extern "C" {
            fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        }

        let cname = CString::new(name).map_err(|_| SymbolError::InvalidName)?;

        unsafe {
            let ptr = dlsym(self.handle.as_ptr(), cname.as_ptr());
            if ptr.is_null() {
                Err(SymbolError::NotFound)
            } else {
                Ok(ptr as *const ())
            }
        }
    }

    #[cfg(windows)]
    fn symbol_impl(&self, name: &str) -> Result<*const (), SymbolError> {
        extern "system" {
            fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
        }

        let cname = CString::new(name).map_err(|_| SymbolError::InvalidName)?;

        unsafe {
            let ptr = GetProcAddress(self.handle.as_ptr(), cname.as_ptr() as *const u8);
            if ptr.is_null() {
                Err(SymbolError::NotFound)
            } else {
                Ok(ptr as *const ())
            }
        }
    }
}

impl Drop for Library {
    #[cfg(unix)]
    fn drop(&mut self) {
        extern "C" {
            fn dlclose(handle: *mut c_void) -> i32;
        }
        unsafe {
            dlclose(self.handle.as_ptr());
        }
    }

    #[cfg(windows)]
    fn drop(&mut self) {
        extern "system" {
            fn FreeLibrary(module: *mut c_void) -> i32;
        }
        unsafe {
            FreeLibrary(self.handle.as_ptr());
        }
    }
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

/// Library loading errors
#[derive(Debug)]
pub enum LoadError {
    InvalidName,
    LoadFailed(String),
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidName => write!(f, "Invalid library name"),
            Self::LoadFailed(msg) => write!(f, "Failed to load library: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// Symbol lookup errors
#[derive(Debug)]
pub enum SymbolError {
    InvalidName,
    NotFound,
}

impl core::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidName => write!(f, "Invalid symbol name"),
            Self::NotFound => write!(f, "Symbol not found"),
        }
    }
}

impl std::error::Error for SymbolError {}

