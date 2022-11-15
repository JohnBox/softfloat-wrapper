//! softfloat-wrapper is a safe wrapper of [Berkeley SoftFloat](https://github.com/ucb-bar/berkeley-softfloat-3) based on [softfloat-sys](https://crates.io/crates/softfloat-sys).
//!
//! ## Examples
//!
//! ```
//! use softfloat_wrapper::{SoftFloat, F16, RoundingMode};
//!
//! fn main() {
//!     let a = 0x1234;
//!     let b = 0x1479;
//!
//!     let a = F16::from_bits(a);
//!     let b = F16::from_bits(b);
//!     let d = a.add(b, RoundingMode::TiesToEven);
//!
//!     let a = f32::from_bits(a.to_f32(RoundingMode::TiesToEven).to_bits());
//!     let b = f32::from_bits(b.to_f32(RoundingMode::TiesToEven).to_bits());
//!     let d = f32::from_bits(d.to_f32(RoundingMode::TiesToEven).to_bits());
//!
//!     println!("{} + {} = {}", a, b, d);
//! }
//! ```

#[cfg(not(feature = "concordium"))]
mod f128;
mod f16;
mod f32;
mod f64;
#[cfg(not(feature = "concordium"))]
pub use crate::f128::F128;
pub use crate::f16::F16;
pub use crate::f32::F32;
pub use crate::f64::F64;

use num_traits::{
    identities::{One, Zero},
    PrimInt,
};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{LowerHex, UpperHex};

pub const DEFAULT_ROUNDING_MODE: RoundingMode = RoundingMode::TiesToAway;
pub const DEFAULT_EXACT_MODE: bool = true;

/// floating-point rounding mode defined by standard
#[derive(Copy, Clone, Debug)]
pub enum RoundingMode {
    /// to nearest, ties to even
    TiesToEven,
    /// toward 0
    TowardZero,
    /// toward −∞
    TowardNegative,
    /// toward +∞
    TowardPositive,
    /// to nearest, ties away from zero
    TiesToAway,
}

impl RoundingMode {
    fn set(&self) {
        unsafe {
            softfloat_sys::softfloat_roundingMode_write_helper(self.to_softfloat());
        }
    }

    fn to_softfloat(&self) -> u8 {
        match self {
            RoundingMode::TiesToEven => softfloat_sys::softfloat_round_near_even,
            RoundingMode::TowardZero => softfloat_sys::softfloat_round_minMag,
            RoundingMode::TowardNegative => softfloat_sys::softfloat_round_min,
            RoundingMode::TowardPositive => softfloat_sys::softfloat_round_max,
            RoundingMode::TiesToAway => softfloat_sys::softfloat_round_near_maxMag,
        }
    }
}

/// exception flags defined by standard
///
/// ## Examples
///
/// ```
/// use softfloat_wrapper::{ExceptionFlags, SoftFloat, RoundingMode, F16};
///
/// let a = 0x0;
/// let b = 0x0;
/// let a = F16::from_bits(a);
/// let b = F16::from_bits(b);
/// let mut flag = ExceptionFlags::default();
/// flag.set();
/// let _d = a.div(b, RoundingMode::TiesToEven);
/// flag.get();
/// assert!(flag.is_invalid());
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct ExceptionFlags(u8);

impl ExceptionFlags {
    const FLAG_INEXACT: u8 = softfloat_sys::softfloat_flag_inexact;
    const FLAG_INFINITE: u8 = softfloat_sys::softfloat_flag_infinite;
    const FLAG_INVALID: u8 = softfloat_sys::softfloat_flag_invalid;
    const FLAG_OVERFLOW: u8 = softfloat_sys::softfloat_flag_overflow;
    const FLAG_UNDERFLOW: u8 = softfloat_sys::softfloat_flag_underflow;

    pub fn from_bits(x: u8) -> Self {
        Self(x)
    }

    pub fn to_bits(&self) -> u8 {
        self.0
    }

    #[deprecated(since = "0.3.0", note = "Please use to_bits instead")]
    pub fn bits(&self) -> u8 {
        self.to_bits()
    }

    pub fn is_inexact(&self) -> bool {
        self.0 & Self::FLAG_INEXACT != 0
    }

    pub fn is_infinite(&self) -> bool {
        self.0 & Self::FLAG_INFINITE != 0
    }

    pub fn is_invalid(&self) -> bool {
        self.0 & Self::FLAG_INVALID != 0
    }

    pub fn is_overflow(&self) -> bool {
        self.0 & Self::FLAG_OVERFLOW != 0
    }

    pub fn is_underflow(&self) -> bool {
        self.0 & Self::FLAG_UNDERFLOW != 0
    }

    pub fn set(&self) {
        unsafe {
            softfloat_sys::softfloat_exceptionFlags_write_helper(self.to_bits());
        }
    }

    pub fn get(&mut self) {
        let x = unsafe { softfloat_sys::softfloat_exceptionFlags_read_helper() };
        self.0 = x;
    }
}

/// arbitrary floting-point type
///
/// ## Examples
///
/// `Float` can be used for generic functions.
///
/// ```
/// use softfloat_wrapper::{SoftFloat, RoundingMode, F16, F32};
///
/// fn rsqrt<T: SoftFloat>(x: T) -> T {
///     let ret = x.sqrt(RoundingMode::TiesToEven);
///     let one = T::from_u8(1, RoundingMode::TiesToEven);
///     one.div(ret, RoundingMode::TiesToEven)
/// }
///
/// let a = F16::from_bits(0x1234);
/// let a = rsqrt(a);
/// let a = F32::from_bits(0x12345678);
/// let a = rsqrt(a);
/// ```
pub trait SoftFloat {
    /// Actual storage type for concrete softfloat implementation
    type Payload: PrimInt + UpperHex + LowerHex;
    /// Mask for mantissa value, starting from 0th bit
    const MANTISSA_MASK: Self::Payload;
    /// Mask for exponent value, starting from 0th bit
    const EXPONENT_MASK: Self::Payload;
    /// Number of mantissa bits, excluding sign
    const MANTISSA_BITS: usize;
    /// Number of exponent bits
    const EXPONENT_BITS: usize;
    /// Sign bit offset
    const SIGN_OFFSET: usize;
    /// Exponent bits offset
    const EXPONENT_OFFSET: usize;

    #[cfg(not(feature = "concordium"))]
    fn from_native_f32(value: f32) -> Self;

    #[cfg(not(feature = "concordium"))]
    fn from_native_f64(value: f64) -> Self;

    fn set_payload(&mut self, x: Self::Payload);

    fn from_bits(v: Self::Payload) -> Self;

    fn to_bits(&self) -> Self::Payload;

    #[deprecated(since = "0.3.0", note = "Please use to_bits instead")]
    fn bits(&self) -> Self::Payload;

    fn add<T: Borrow<Self>>(&self, x: T, rnd: RoundingMode) -> Self;

    fn sub<T: Borrow<Self>>(&self, x: T, rnd: RoundingMode) -> Self;

    fn mul<T: Borrow<Self>>(&self, x: T, rnd: RoundingMode) -> Self;

    fn fused_mul_add<T: Borrow<Self>>(&self, x: T, y: T, rnd: RoundingMode) -> Self;

    fn div<T: Borrow<Self>>(&self, x: T, rnd: RoundingMode) -> Self;

    fn rem<T: Borrow<Self>>(&self, x: T, rnd: RoundingMode) -> Self;

    fn sqrt(&self, rnd: RoundingMode) -> Self;

    fn eq<T: Borrow<Self>>(&self, x: T) -> bool;

    fn lt<T: Borrow<Self>>(&self, x: T) -> bool;

    fn le<T: Borrow<Self>>(&self, x: T) -> bool;

    fn lt_quiet<T: Borrow<Self>>(&self, x: T) -> bool;

    fn le_quiet<T: Borrow<Self>>(&self, x: T) -> bool;

    fn eq_signaling<T: Borrow<Self>>(&self, x: T) -> bool;

    fn is_signaling_nan(&self) -> bool;

    fn from_u32(x: u32, rnd: RoundingMode) -> Self;

    fn from_u64(x: u64, rnd: RoundingMode) -> Self;

    fn from_i32(x: i32, rnd: RoundingMode) -> Self;

    fn from_i64(x: i64, rnd: RoundingMode) -> Self;

    fn to_u32(&self, rnd: RoundingMode, exact: bool) -> u32;

    fn to_u64(&self, rnd: RoundingMode, exact: bool) -> u64;

    fn to_i32(&self, rnd: RoundingMode, exact: bool) -> i32;

    fn to_i64(&self, rnd: RoundingMode, exact: bool) -> i64;

    fn to_f16(&self, rnd: RoundingMode) -> F16;

    fn to_f32(&self, rnd: RoundingMode) -> F32;

    fn to_f64(&self, rnd: RoundingMode) -> F64;

    #[cfg(not(feature = "concordium"))]
    fn to_f128(&self, rnd: RoundingMode) -> F128;

    fn round_to_integral(&self, rnd: RoundingMode) -> Self;

    #[inline]
    fn compare<T: Borrow<Self>>(&self, x: T) -> Option<Ordering> {
        let eq = self.eq(x.borrow());
        let lt = self.lt(x.borrow());
        if self.is_nan() || x.borrow().is_nan() {
            None
        } else if eq {
            Some(Ordering::Equal)
        } else if lt {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }

    #[inline]
    fn from_u8(x: u8, rnd: RoundingMode) -> Self
    where
        Self: Sized,
    {
        Self::from_u32(x as u32, rnd)
    }

    #[inline]
    fn from_u16(x: u16, rnd: RoundingMode) -> Self
    where
        Self: Sized,
    {
        Self::from_u32(x as u32, rnd)
    }

    #[inline]
    fn from_i8(x: i8, rnd: RoundingMode) -> Self
    where
        Self: Sized,
    {
        Self::from_i32(x as i32, rnd)
    }

    #[inline]
    fn from_i16(x: i16, rnd: RoundingMode) -> Self
    where
        Self: Sized,
    {
        Self::from_i32(x as i32, rnd)
    }

    #[inline]
    fn neg(&self) -> Self
    where
        Self: Sized,
    {
        let mut ret = Self::from_bits(self.to_bits());
        ret.set_sign(!self.sign());
        ret
    }

    #[inline]
    fn abs(&self) -> Self
    where
        Self: Sized,
    {
        let mut ret = Self::from_bits(self.to_bits());
        ret.set_sign(Self::Payload::zero());
        ret
    }

    #[inline]
    fn sign(&self) -> Self::Payload {
        (self.to_bits() >> Self::SIGN_OFFSET) & Self::Payload::one()
    }

    #[inline]
    fn exponent(&self) -> Self::Payload {
        (self.to_bits() >> Self::EXPONENT_OFFSET) & Self::EXPONENT_MASK
    }

    #[inline]
    fn mantissa(&self) -> Self::Payload {
        self.to_bits() & Self::MANTISSA_MASK
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.sign() == Self::Payload::zero()
    }

    #[inline]
    fn is_positive_zero(&self) -> bool {
        self.is_positive()
            && self.exponent() == Self::Payload::zero()
            && self.mantissa() == Self::Payload::zero()
    }

    #[inline]
    fn is_positive_subnormal(&self) -> bool {
        self.is_positive()
            && self.exponent() == Self::Payload::zero()
            && self.mantissa() != Self::Payload::zero()
    }

    #[inline]
    fn is_positive_normal(&self) -> bool {
        self.is_positive()
            && self.exponent() != Self::Payload::zero()
            && self.exponent() != Self::EXPONENT_MASK
    }

    #[inline]
    fn is_positive_infinity(&self) -> bool {
        self.is_positive()
            && self.exponent() == Self::EXPONENT_MASK
            && self.mantissa() == Self::Payload::zero()
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.sign() == Self::Payload::one()
    }

    #[inline]
    fn is_negative_zero(&self) -> bool {
        self.is_negative()
            && self.exponent() == Self::Payload::zero()
            && self.mantissa() == Self::Payload::zero()
    }

    #[inline]
    fn is_negative_subnormal(&self) -> bool {
        self.is_negative()
            && self.exponent() == Self::Payload::zero()
            && self.mantissa() != Self::Payload::zero()
    }

    #[inline]
    fn is_negative_normal(&self) -> bool {
        self.is_negative()
            && self.exponent() != Self::Payload::zero()
            && self.exponent() != Self::EXPONENT_MASK
    }

    #[inline]
    fn is_negative_infinity(&self) -> bool {
        self.is_negative()
            && self.exponent() == Self::EXPONENT_MASK
            && self.mantissa() == Self::Payload::zero()
    }

    #[inline]
    fn is_nan(&self) -> bool {
        self.exponent() == Self::EXPONENT_MASK && self.mantissa() != Self::Payload::zero()
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.is_positive_zero() || self.is_negative_zero()
    }

    #[inline]
    fn is_subnormal(&self) -> bool {
        self.exponent() == Self::Payload::zero()
    }

    #[inline]
    fn set_sign(&mut self, x: Self::Payload) {
        self.set_payload(
            (self.to_bits() & !(Self::Payload::one() << Self::SIGN_OFFSET))
                | ((x & Self::Payload::one()) << Self::SIGN_OFFSET),
        );
    }

    #[inline]
    fn set_exponent(&mut self, x: Self::Payload) {
        self.set_payload(
            (self.to_bits() & !(Self::EXPONENT_MASK << Self::EXPONENT_OFFSET))
                | ((x & Self::EXPONENT_MASK) << Self::EXPONENT_OFFSET),
        );
    }

    #[inline]
    fn set_mantissa(&mut self, x: Self::Payload) {
        self.set_payload((self.to_bits() & !Self::MANTISSA_MASK) | (x & Self::MANTISSA_MASK));
    }

    #[inline]
    fn positive_infinity() -> Self
    where
        Self: Sized,
    {
        let mut x = Self::from_bits(Self::Payload::zero());
        x.set_exponent(Self::EXPONENT_MASK);
        x
    }

    #[inline]
    fn positive_zero() -> Self
    where
        Self: Sized,
    {
        let x = Self::from_bits(Self::Payload::zero());
        x
    }

    #[inline]
    fn negative_infinity() -> Self
    where
        Self: Sized,
    {
        let mut x = Self::from_bits(Self::Payload::zero());
        x.set_sign(Self::Payload::one());
        x.set_exponent(Self::EXPONENT_MASK);
        x
    }

    #[inline]
    fn negative_zero() -> Self
    where
        Self: Sized,
    {
        let mut x = Self::from_bits(Self::Payload::zero());
        x.set_sign(Self::Payload::one());
        x
    }

    #[inline]
    fn quiet_nan() -> Self
    where
        Self: Sized,
    {
        let mut x = Self::from_bits(Self::Payload::zero());
        x.set_exponent(Self::EXPONENT_MASK);
        x.set_mantissa(Self::Payload::one() << (Self::EXPONENT_OFFSET - 1));
        x
    }
}

macro_rules! impl_ops {
    ($type:ident) => {
        #[cfg(feature = "concordium")]
        impl concordium_std::schema::SchemaType for $type {
            fn get_type() -> concordium_std::schema::Type {
                <<Self as crate::SoftFloat>::Payload as concordium_std::schema::SchemaType>::get_type()
            }
        }

        #[cfg(feature = "concordium")]
        impl concordium_std::Serial for $type {
            fn serial<W: concordium_std::Write>(&self, out: &mut W) -> Result<(), W::Err> {
                self.to_bits().serial(out)
            }
        }

        #[cfg(feature = "concordium")]
        impl concordium_std::Deserial for $type {
            fn deserial<R: concordium_std::Read>(source: &mut R) -> concordium_std::ParseResult<Self> {
                Ok(Self::from_bits(
                    <Self as crate::SoftFloat>::Payload::deserial(source)?,
                ))
            }
        }

        impl Default for $type {
            fn default() -> Self {
                num_traits::Zero::zero()
            }
        }

        impl num_traits::Zero for $type {
            fn zero() -> Self {
                crate::SoftFloat::positive_zero()
            }

            fn is_zero(&self) -> bool {
                crate::SoftFloat::is_zero(self)
            }
        }

        impl num_traits::One for $type {
            fn one() -> Self {
                crate::SoftFloat::from_i8(1, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::Neg for $type {
            type Output = Self;

            fn neg(self) -> Self::Output {
                crate::SoftFloat::neg(&self)
            }
        }

        impl std::ops::Add for $type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                crate::SoftFloat::add(&self, rhs, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl std::ops::Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                crate::SoftFloat::sub(&self, rhs, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl std::ops::Mul for $type {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                crate::SoftFloat::mul(&self, rhs, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::MulAssign for $type {
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl std::ops::Div for $type {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                crate::SoftFloat::div(&self, rhs, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::DivAssign for $type {
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl std::ops::Rem for $type {
            type Output = Self;

            fn rem(self, rhs: Self) -> Self::Output {
                crate::SoftFloat::rem(&self, rhs, crate::DEFAULT_ROUNDING_MODE)
            }
        }

        impl std::ops::RemAssign for $type {
            fn rem_assign(&mut self, rhs: Self) {
                *self = *self % rhs;
            }
        }

        impl std::cmp::PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                crate::SoftFloat::eq(self, other)
            }
        }

        impl std::cmp::PartialOrd for $type {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                crate::SoftFloat::compare(self, other)
            }
        }
    };
}

impl_ops!(F16);
impl_ops!(F32);
impl_ops!(F64);
#[cfg(not(feature = "concordium"))]
impl_ops!(F128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_inexact() {
        let a = 0x1234;
        let b = 0x7654;
        let a = F16::from_bits(a);
        let b = F16::from_bits(b);
        let mut flag = ExceptionFlags::default();
        flag.set();
        let _d = a.add(b, RoundingMode::TiesToEven);
        flag.get();
        assert!(flag.is_inexact());
        assert!(!flag.is_infinite());
        assert!(!flag.is_invalid());
        assert!(!flag.is_overflow());
        assert!(!flag.is_underflow());
    }

    #[test]
    fn flag_infinite() {
        let a = 0x1234;
        let b = 0x0;
        let a = F16::from_bits(a);
        let b = F16::from_bits(b);
        let mut flag = ExceptionFlags::default();
        flag.set();
        let _d = a.div(b, RoundingMode::TiesToEven);
        flag.get();
        assert!(!flag.is_inexact());
        assert!(flag.is_infinite());
        assert!(!flag.is_invalid());
        assert!(!flag.is_overflow());
        assert!(!flag.is_underflow());
    }

    #[test]
    fn flag_invalid() {
        let a = 0x0;
        let b = 0x0;
        let a = F16::from_bits(a);
        let b = F16::from_bits(b);
        let mut flag = ExceptionFlags::default();
        flag.set();
        let _d = a.div(b, RoundingMode::TiesToEven);
        flag.get();
        assert!(!flag.is_inexact());
        assert!(!flag.is_infinite());
        assert!(flag.is_invalid());
        assert!(!flag.is_overflow());
        assert!(!flag.is_underflow());
    }

    #[test]
    fn flag_overflow() {
        let a = 0x7bff;
        let b = 0x7bff;
        let a = F16::from_bits(a);
        let b = F16::from_bits(b);
        let mut flag = ExceptionFlags::default();
        flag.set();
        let _d = a.add(b, RoundingMode::TiesToEven);
        flag.get();
        assert!(flag.is_inexact());
        assert!(!flag.is_infinite());
        assert!(!flag.is_invalid());
        assert!(flag.is_overflow());
        assert!(!flag.is_underflow());
    }

    #[test]
    fn flag_underflow() {
        let a = 0x0001;
        let b = 0x0001;
        let a = F16::from_bits(a);
        let b = F16::from_bits(b);
        let mut flag = ExceptionFlags::default();
        flag.set();
        let _d = a.mul(b, RoundingMode::TiesToEven);
        flag.get();
        assert!(flag.is_inexact());
        assert!(!flag.is_infinite());
        assert!(!flag.is_invalid());
        assert!(!flag.is_overflow());
        assert!(flag.is_underflow());
    }
}
