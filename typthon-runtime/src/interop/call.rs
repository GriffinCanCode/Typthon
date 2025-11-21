//! Dynamic function calling with multiple calling conventions
//!
//! Implements architecture-specific function invocation.

use super::abi::{CallingConvention, RegisterAllocator};
use super::types::{FfiType, FfiValue, TypedArg};

/// Function call descriptor
pub struct FunctionCall {
    ptr: *const (),
    convention: CallingConvention,
    return_type: FfiType,
    arg_types: Vec<FfiType>,
}

impl FunctionCall {
    /// Create function call descriptor
    #[inline]
    pub fn new(
        ptr: *const (),
        convention: CallingConvention,
        return_type: FfiType,
        arg_types: Vec<FfiType>,
    ) -> Self {
        Self {
            ptr,
            convention,
            return_type,
            arg_types,
        }
    }

    /// Call function with arguments
    ///
    /// # Safety
    /// Caller must ensure:
    /// - Function pointer is valid
    /// - Arguments match declared types
    /// - Calling convention matches function
    pub unsafe fn call(&self, args: &[FfiValue]) -> Result<FfiValue, CallError> {
        if args.len() != self.arg_types.len() {
            return Err(CallError::ArgCountMismatch {
                expected: self.arg_types.len(),
                got: args.len(),
            });
        }

        // Platform-specific implementation
        self.call_impl(args)
    }

    /// Platform-specific call implementation
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    unsafe fn call_impl(&self, args: &[FfiValue]) -> Result<FfiValue, CallError> {
        // Allocate register tracker
        let mut allocator = RegisterAllocator::new(self.convention);

        // Prepare arguments (register vs stack)
        let mut stack_args = Vec::new();

        for (i, &arg) in args.iter().enumerate() {
            let ty = self.arg_types[i];
            let is_float = ty.is_float();

            if allocator.can_use_register(is_float) {
                allocator.use_register(is_float);
                // Register args handled by inline assembly
            } else {
                // Stack args need explicit push
                stack_args.push(arg);
            }
        }

        // Architecture-specific inline assembly call
        self.call_asm(args, &stack_args)
    }

    /// Assembly-level function call
    #[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
    unsafe fn call_asm(
        &self,
        args: &[FfiValue],
        _stack_args: &[FfiValue],
    ) -> Result<FfiValue, CallError> {
        // System V x86-64 calling convention
        // Args in: RDI, RSI, RDX, RCX, R8, R9, then stack
        // Return in: RAX (int) or XMM0 (float)

        let ret_val: u64;
        let arg_count = args.len().min(6);

        match arg_count {
            0 => {
                core::arch::asm!(
                    "call {func}",
                    func = in(reg) self.ptr,
                    lateout("rax") ret_val,
                    clobber_abi("C"),
                );
            }
            1 => {
                core::arch::asm!(
                    "call {func}",
                    func = in(reg) self.ptr,
                    in("rdi") args[0].i64,
                    lateout("rax") ret_val,
                    clobber_abi("C"),
                );
            }
            2 => {
                core::arch::asm!(
                    "call {func}",
                    func = in(reg) self.ptr,
                    in("rdi") args[0].i64,
                    in("rsi") args[1].i64,
                    lateout("rax") ret_val,
                    clobber_abi("C"),
                );
            }
            _ => {
                // For 3+ args, use generic path
                return Err(CallError::TooManyArgs);
            }
        }

        Ok(FfiValue { i64: ret_val as i64 })
    }

    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    unsafe fn call_asm(
        &self,
        args: &[FfiValue],
        _stack_args: &[FfiValue],
    ) -> Result<FfiValue, CallError> {
        // Windows x64 calling convention
        // Args in: RCX, RDX, R8, R9, then stack
        // Return in: RAX (int) or XMM0 (float)

        let ret_val: u64;
        let arg_count = args.len().min(4);

        match arg_count {
            0 => {
                core::arch::asm!(
                    "call {func}",
                    func = in(reg) self.ptr,
                    lateout("rax") ret_val,
                    clobber_abi("C"),
                );
            }
            1 => {
                core::arch::asm!(
                    "call {func}",
                    func = in(reg) self.ptr,
                    in("rcx") args[0].i64,
                    lateout("rax") ret_val,
                    clobber_abi("C"),
                );
            }
            _ => {
                return Err(CallError::TooManyArgs);
            }
        }

        Ok(FfiValue { i64: ret_val as i64 })
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn call_asm(
        &self,
        args: &[FfiValue],
        _stack_args: &[FfiValue],
    ) -> Result<FfiValue, CallError> {
        // ARM64 calling convention
        // Args in: X0-X7, then stack
        // Return in: X0 (int) or D0 (float)

        let ret_val: u64;
        let arg_count = args.len().min(8);

        match arg_count {
            0 => {
                core::arch::asm!(
                    "blr {func}",
                    func = in(reg) self.ptr,
                    lateout("x0") ret_val,
                    clobber_abi("C"),
                );
            }
            1 => {
                core::arch::asm!(
                    "blr {func}",
                    func = in(reg) self.ptr,
                    in("x0") args[0].i64,
                    lateout("x0") ret_val,
                    clobber_abi("C"),
                );
            }
            _ => {
                return Err(CallError::TooManyArgs);
            }
        }

        Ok(FfiValue { i64: ret_val as i64 })
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    unsafe fn call_impl(&self, _args: &[FfiValue]) -> Result<FfiValue, CallError> {
        // Fallback for unsupported architectures
        Err(CallError::UnsupportedArchitecture)
    }
}

/// Function call errors
#[derive(Debug)]
pub enum CallError {
    ArgCountMismatch { expected: usize, got: usize },
    TooManyArgs,
    UnsupportedArchitecture,
}

impl core::fmt::Display for CallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArgCountMismatch { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
            Self::TooManyArgs => write!(f, "Too many arguments for inline call"),
            Self::UnsupportedArchitecture => write!(f, "Architecture not supported"),
        }
    }
}

impl std::error::Error for CallError {}

/// High-level API: Call C function with typed arguments
///
/// # Safety
/// See `FunctionCall::call` safety requirements
#[inline]
pub unsafe fn call_extern(
    fn_ptr: *const (),
    args: &[TypedArg],
    return_type: FfiType,
) -> Result<FfiValue, CallError> {
    let arg_types: Vec<_> = args.iter().map(|a| a.ty).collect();
    let arg_values: Vec<_> = args.iter().map(|a| a.value).collect();

    let call = FunctionCall::new(
        fn_ptr,
        CallingConvention::default(),
        return_type,
        arg_types,
    );

    call.call(&arg_values)
}

