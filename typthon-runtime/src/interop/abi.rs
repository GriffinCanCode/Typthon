//! ABI (Application Binary Interface) handling
//!
//! Supports multiple calling conventions across architectures.

/// Calling convention specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CallingConvention {
    /// C calling convention (platform default)
    C,
    /// System V AMD64 ABI (Unix x86-64)
    SysV,
    /// Microsoft x64 calling convention (Windows)
    Win64,
    /// ARM AAPCS (ARM 32-bit)
    Aapcs,
    /// ARM64 calling convention
    Aarch64,
}

impl CallingConvention {
    /// Get platform default
    #[inline]
    pub const fn default() -> Self {
        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        return Self::Win64;

        #[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
        return Self::SysV;

        #[cfg(target_arch = "aarch64")]
        return Self::Aarch64;

        #[cfg(target_arch = "arm")]
        return Self::Aapcs;

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "arm"
        )))]
        return Self::C;
    }

    /// Maximum register arguments for this convention
    #[inline]
    pub const fn max_register_args(self) -> usize {
        match self {
            Self::C => 6,
            Self::SysV => 6,    // RDI, RSI, RDX, RCX, R8, R9
            Self::Win64 => 4,   // RCX, RDX, R8, R9
            Self::Aapcs => 4,   // R0-R3
            Self::Aarch64 => 8, // X0-X7
        }
    }

    /// Check if floating-point args use separate registers
    #[inline]
    pub const fn has_fp_registers(self) -> bool {
        matches!(self, Self::SysV | Self::Win64 | Self::Aarch64)
    }
}

impl Default for CallingConvention {
    #[inline]
    fn default() -> Self {
        Self::default()
    }
}

/// Register allocation strategy for function calls
pub struct RegisterAllocator {
    convention: CallingConvention,
    int_regs_used: usize,
    fp_regs_used: usize,
}

impl RegisterAllocator {
    /// Create allocator for calling convention
    #[inline]
    pub const fn new(convention: CallingConvention) -> Self {
        Self {
            convention,
            int_regs_used: 0,
            fp_regs_used: 0,
        }
    }

    /// Check if next arg goes in register
    #[inline]
    pub fn can_use_register(&mut self, is_float: bool) -> bool {
        if is_float && self.convention.has_fp_registers() {
            self.fp_regs_used < self.convention.max_register_args()
        } else {
            self.int_regs_used < self.convention.max_register_args()
        }
    }

    /// Mark register as used
    #[inline]
    pub fn use_register(&mut self, is_float: bool) {
        if is_float && self.convention.has_fp_registers() {
            self.fp_regs_used += 1;
        } else {
            self.int_regs_used += 1;
        }
    }

    /// Reset for new call
    #[inline]
    pub fn reset(&mut self) {
        self.int_regs_used = 0;
        self.fp_regs_used = 0;
    }
}

