#![cfg_attr(not(feature = "emulate"), no_std)]
#![cfg_attr(not(feature = "emulate"), feature(wasm_simd, simd_ffi))]
#![feature(min_const_generics)]

#[cfg(feature = "emulate")]
pub use std::process::exit;

scale_impl_generator::impls!();

#[cfg(not(feature = "emulate"))]
mod types {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct ClearModp(pub core::arch::wasm32::v128);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SecretModp(f32);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SecretI64(f64);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SecretBit(pub(crate) SecretModp);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct RawSecretBit(pub(crate) f64);
}

#[cfg(feature = "emulate")]
mod types {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct ClearModp(pub(crate) i64);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SecretModp(pub(crate) ClearModp);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SecretI64(pub(crate) i64);
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    // FIXME: use `bool` as the field
    pub struct SecretBit(pub(crate) i64);
    // Due to the fact that our rust -> wasm -> scasm pipeline
    // does not support `externref` yet, so we can't properly
    // encode more than 4 different register types, we need
    // to have this hacky datastructure that the raw asm
    // instructions use.
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct RawSecretBit(pub(crate) i64);
}

pub use types::*;

macro_rules! any_register {
    ($($bound:path),*) => {
        /// A helper trait that is implemented for all register types
        /// and has bounds for all operations implemented by those types.
        /// This allows one to easily write code that is generic over
        /// the various register types.
        pub trait AnyRegister: $($bound+)* Sized {}
    };
}

impl AnyRegister for ClearModp {}
impl AnyRegister for SecretModp {}
impl AnyRegister for i64 {}
impl AnyRegister for SecretI64 {}

use core::ops::*;

any_register! {
    Stack,
    alloc::GetAllocator,
    Add<Output = Self>,
    Sub<Output = Self>,
    Mul<Output = Self>,
    StoreInMem<i64>,
    LoadFromMem<i64>
}

pub trait InnerModp:
    AnyRegister
    + From<ClearModp>
    + Into<SecretModp>
    + RevealIfSecret
    + Mul<ClearModp, Output = Self>
    + Add<ClearModp, Output = Self>
    + Sub<ClearModp, Output = Self>
    + Mul<SecretModp, Output = SecretModp>
    + Add<SecretModp, Output = SecretModp>
    + Sub<SecretModp, Output = SecretModp>
{
}

impl InnerModp for ClearModp {}
impl InnerModp for SecretModp {}

pub trait Modp<Other: InnerModp>:
    InnerModp
    + Mul<Other, Output = SecretModp>
    + Add<Other, Output = SecretModp>
    + Sub<Other, Output = SecretModp>
{
}

impl Modp<ClearModp> for SecretModp {}
impl Modp<SecretModp> for ClearModp {}
impl Modp<SecretModp> for SecretModp {}

pub trait StoreInMem<Index> {
    fn store_in_mem(&self, idx: Index);
}

pub trait LoadFromMem<Index> {
    fn load_from_mem(idx: Index) -> Self;
}

impl<T: StoreInMem<i64>> StoreInMem<(i64, i64)> for Option<T> {
    fn store_in_mem(&self, idk: (i64, i64)) {
        match self {
            None => 0_i64.store_in_mem(idk.0),
            Some(a) => {
                1_i64.store_in_mem(idk.0);
                a.store_in_mem(idk.1);
            }
        };
    }
}

impl<T: LoadFromMem<i64>> LoadFromMem<(i64, i64)> for Option<T> {
    fn load_from_mem(idk: (i64, i64)) -> Option<T> {
        let ind = i64::load_from_mem(idk.0);
        return match ind {
            1 => Some(T::load_from_mem(idk.1)),
            _ => None,
        };
    }
}

mod stack_address;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct StackAddress(i64);

pub trait Stack {
    /// Push a new item to the stack
    ///
    /// SAFETY: this function is unsafe, because runtime function calls also
    /// generate these instructions, and thus modifying the stack must be done
    /// very carefully.
    unsafe fn push(val: &Self);
    /// Read and remove the top item from the stack
    ///
    /// SAFETY: this function is unsafe, because runtime function calls also
    /// generate these instructions, and thus modifying the stack must be done
    /// very carefully.
    unsafe fn pop() -> Self;
    /// Read an item at the given stack address
    unsafe fn peek(addr: StackAddress) -> Self;
    /// Replace the item at the given stack address
    ///
    /// SAFETY: this function is unsafe, because runtime function calls also
    /// generate these instructions, and thus modifying the stack must be done
    /// very carefully.
    unsafe fn poke(addr: StackAddress, val: &Self);
    /// Get the stack address of the top most element on the stack
    fn get_stack_pointer() -> StackAddress;
    /// Get the value at the stack address "stackpointer - offset"
    unsafe fn peek_from_top(offset: i64) -> Self;
    /// Overwrite the value at the stack address "stackpointer - offset"
    unsafe fn poke_from_top(offset: i64, val: &Self);
}

use core::mem::MaybeUninit;

impl<T: alloc::GetAllocator + Stack, const N: usize> Stack for [T; N] {
    #[inline(always)]
    unsafe fn push(array: &Self) {
        for val in array.iter() {
            T::push(&val);
        }
    }
    #[inline(always)]
    unsafe fn pop() -> Self {
        let mut array: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
        for i in (0..N).rev() {
            array[i] = MaybeUninit::new(T::pop());
        }
        core::mem::transmute_copy(&array)
    }

    #[inline(always)]
    unsafe fn peek(addr: StackAddress) -> Self {
        let mut array: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            array[i] = MaybeUninit::new(T::peek(addr + i as _));
        }
        core::mem::transmute_copy(&array)
    }

    #[inline(always)]
    unsafe fn poke(addr: StackAddress, array: &Self) {
        for (i, val) in array.iter().enumerate() {
            T::poke(addr + i as _, &val);
        }
    }

    #[inline(always)]
    unsafe fn peek_from_top(offset: i64) -> Self {
        let mut array: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            array[i] = MaybeUninit::new(T::peek_from_top(offset + i as i64));
        }
        core::mem::transmute_copy(&array)
    }

    #[inline(always)]
    unsafe fn poke_from_top(offset: i64, array: &Self) {
        for (i, val) in array.iter().enumerate() {
            T::poke_from_top(offset + i as i64, &val);
        }
    }

    #[inline(always)]
    fn get_stack_pointer() -> StackAddress {
        panic!("this operation makes no sense")
    }
}

pub trait Reveal {
    type Output;
    fn reveal(&self) -> Self::Output;
}

pub trait RevealIfSecret {
    type Output;
    fn reveal_if_secret(&self) -> Self::Output;
}

mod clear_modp;
mod i64;
mod io;
mod secret_bit;
mod secret_i64;
mod secret_modp;
mod system;

pub use io::*;
pub use system::*;

#[cfg(not(feature = "emulate"))]
extern "Rust" {
    fn __black_box(i: i64) -> i64;
    /// This is a helper doing `startopen` and `stopopen` at the same time for a single register.
    /// We don't have a setup for passing arrays to assembly yet, so we do this for now.
    pub fn reveal(s: SecretModp) -> ClearModp;
    /// A helper for multi-return-value instructions to extract the next instruction
    fn pop_secret_modp() -> SecretModp;
    /// A helper for multi-return-value instructions to extract the next instruction
    fn pop_secret_bit() -> RawSecretBit;
    /// A helper for multi-return-value instructions to extract the next instruction
    fn pop_secret_i64() -> SecretI64;
}

#[no_mangle]
pub static mut I64_MEMORY: i64 = 0;
#[no_mangle]
pub static mut SECRET_I64_MEMORY: i64 = 0;
#[no_mangle]
pub static mut CLEAR_MODP_MEMORY: i64 = 0;
#[no_mangle]
pub static mut SECRET_MODP_MEMORY: i64 = 0;
#[no_mangle]
pub static mut KAPPA: u64 = 40;

/// We reserve 1000 memory entries for the stack.
/// FIXME: This is an arbitrary choice in order to put
/// `test()` dumps after the local variables.
const TEST_MEMORY_OFFSET: i64 = 1000;

#[cfg(feature = "emulate")]
#[inline(never)]
pub fn black_box(i: i64) -> i64 {
    i
}

#[cfg(not(feature = "emulate"))]
#[inline(always)]
/// This function prevents wasm-opt from optimizing out operations on the value
/// if said value is constant or dead code.
pub fn black_box(i: i64) -> i64 {
    unsafe { __black_box(i) }
}

#[cfg(feature = "emulate")]
#[inline(never)]
pub unsafe fn reveal(s: SecretModp) -> ClearModp {
    s.0
}

#[panic_handler]
#[cfg(not(feature = "emulate"))]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    /*
    // FIXME: this requires some fancy memory operations to work
    if let Some(loc) = info.location() {
        print!("panic in line ", i64::from(loc.line()));
    }*/
    unsafe { __crash() }
}

#[cfg(feature = "emulate")]
mod emulated_impls;

#[cfg(feature = "emulate")]
pub use emulated_impls::__mul2sint;

#[cfg(feature = "emulate")]
use num_bigint::BigInt;

#[cfg(feature = "emulate")]
// __triple and __square are here, because they have multiple return values and this isn't supported properly
// in our wasm transpiler. Look at the code generator for these to see how wasm handles it.
pub fn __triple() -> (SecretModp, SecretModp, SecretModp) {
    let value: BigInt = From::from(1);
    let s = SecretModp(value.into());
    (s, s, s)
}

#[cfg(feature = "emulate")]
pub fn __square() -> (SecretModp, SecretModp) {
    let value: BigInt = From::from(1);
    let s = SecretModp(value.into());
    (s, s)
}

pub trait Test {
    #[track_caller]
    fn test(self);
}

pub trait TestValue {
    #[track_caller]
    fn test_value(self, val: Self);
}

// Note test_mem is never implemented for secret types
//   - This makes things a bit simpler
pub trait TestMem {
    #[track_caller]
    fn test_mem<const I: u32>(self, t_location: ConstU32<I>);
}

pub trait Output {
    fn output<const C: u32>(self, ch: Channel<C>);
}

pub trait Input {
    fn input<const C: u32>(ch: Channel<C>) -> Self;
}

#[cfg(not(feature = "emulate"))]
mod testing;

#[cfg(feature = "emulate")]
mod testing_emulated;

pub mod alloc;

#[macro_export]
macro_rules! main {
    (I64_MEMORY = $a:expr;SECRET_I64_MEMORY = $b:expr;SECRET_MODP_MEMORY = $c:expr;CLEAR_MODP_MEMORY = $d:expr;KAPPA = $e:expr;) => {
        use $crate::print;
        use $crate::*;
        mod helper {
            #[no_mangle]
            fn main() {
                extern "Rust" {
                    fn set_i64_max_memory(n: u32);
                    fn set_secret_i64_max_memory(n: u32);
                    fn set_clear_modp_max_memory(n: u32);
                    fn set_secret_modp_max_memory(n: u32);
                }
                use $crate::alloc::{Allocate, GetAllocator};
                // This order is important, the first instruction before any memory access ever must be `set_i64_max_memory`
                unsafe {
                    set_i64_max_memory($a);
                    set_secret_i64_max_memory($b);
                    set_secret_modp_max_memory($c);
                    set_clear_modp_max_memory($d);
                }
                if $a > 0 {
                    unsafe {
                        $crate::I64_MEMORY = $a;
                        i64::get_allocator().free($a);
                    }
                }
                if $b > 0 {
                    unsafe {
                        $crate::SECRET_I64_MEMORY = $b;
                        $crate::SecretI64::get_allocator().free($b);
                    }
                }
                if $c > 0 {
                    unsafe {
                        $crate::SECRET_MODP_MEMORY = $c;
                        $crate::SecretModp::get_allocator().free($c);
                    }
                }
                if $d > 0 {
                    unsafe {
                        $crate::CLEAR_MODP_MEMORY = $d;
                        $crate::ClearModp::get_allocator().free($d);
                    }
                }
                if $e > 0 {
                    unsafe {
                        $crate::KAPPA = $d;
                    }
                }
                super::main();
                #[cfg(feature = "emulate")]
                $crate::exit(0);
            }
        }
    };
}

pub trait Print {
    fn print(self);
}

impl Print for &'static str {
    #[inline(always)]
    fn print(self) {
        for &c in self.as_bytes() {
            unsafe { __print_char_regint(c.into()) }
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($e:expr),*) => {
        $($crate::Print::print($e);)*
    }
}

#[macro_export]
macro_rules! panic {
    ($($tt:tt)*) => {{
        $crate::print!($($tt)*);
        unsafe { $crate::__crash() }
    }};
}

#[macro_export]
macro_rules! assert {
    ($e:expr) => {
        if !$e {
            $crate::panic!("assertion failed: ", stringify!($e));
        }
    };
}

pub trait SecretCmp<Other> {
    fn lt(self, other: Other) -> SecretBit;
    fn le(self, other: Other) -> SecretBit;
    fn gt(self, other: Other) -> SecretBit;
    fn ge(self, other: Other) -> SecretBit;
    fn eq(self, other: Other) -> SecretBit;
    fn ne(self, other: Other) -> SecretBit;
}

pub trait SecretCmpZ {
    fn ltz(self) -> SecretBit;
    fn lez(self) -> SecretBit;
    fn gtz(self) -> SecretBit;
    fn gez(self) -> SecretBit;
    fn eqz(self) -> SecretBit;
    fn nez(self) -> SecretBit;
}

pub trait Rand {
    fn rand(self) -> i64;
}

#[macro_export]
macro_rules! execute_garbled_circuit {
    ($id:ident($($arg:expr),*) -> $($ret:ty),*) => {{
        $(Stack::push(&$arg);)*
        execute_garbled_circuit(ConstU32::<$id>);
        ($(<$ret>::pop()),*)
    }};
}

#[macro_export]
macro_rules! execute_local_function {
    ($id:ident($($arg:expr),*) -> $($ret:ty),*) => {{
        $(Stack::push(&$arg);)*
        execute_local_function(ConstU32::<$id>);
        ($(<$ret>::pop()),*)
    }};
}

#[derive(Copy, Clone)]
pub struct Player<const ID: u32>;

#[derive(Copy, Clone)]
pub struct ConstU32<const U: u32>;

#[derive(Copy, Clone)]
pub struct ConstI32<const I: i32>;

#[derive(Copy, Clone)]
pub struct Channel<const ID: u32>;

/// Internal helper to map user-visible types
/// to internal types.
trait AssemblyType {
    type Type;
    fn to_asm(self) -> Self::Type;
}

impl AssemblyType for i64 {
    type Type = i64;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

impl AssemblyType for ClearModp {
    type Type = ClearModp;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

impl AssemblyType for SecretModp {
    type Type = SecretModp;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

impl AssemblyType for SecretBit {
    type Type = SecretBit;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

impl AssemblyType for SecretI64 {
    type Type = SecretI64;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

impl<const I: u32> AssemblyType for Channel<I> {
    type Type = u32;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        I
    }
}

impl<const I: i32> AssemblyType for ConstI32<I> {
    type Type = i32;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        I
    }
}

impl<const I: u32> AssemblyType for ConstU32<I> {
    type Type = u32;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        I
    }
}

impl<const I: u32> AssemblyType for Player<I> {
    type Type = u32;

    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        I
    }
}

impl AssemblyType for Never {
    type Type = Never;
    #[inline(always)]
    fn to_asm(self) -> Self::Type {
        self
    }
}

type Never = <F as HasOutput>::Output;
/// Helper type giving us access to the `!` type on the stable compiler
pub trait HasOutput {
    type Output;
}

impl<O> HasOutput for fn() -> O {
    type Output = O;
}

type F = fn() -> !;
