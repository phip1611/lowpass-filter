/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Numeric traits which enables this library to use any combination of primitive
//! input data types. The popular `num-complex`-crate didn't provide what I wanted.
//! THerefore I

use core::f32;
use core::f64;
use core::ops::{Add, Div, Mul, Sub};

/// This traits shall be implemented for all primitive numeric data types.
/// It enables us to input any numeric literal or variable we want into the function,
/// and the function just works.
/// It is similar to [`From`] but it enables a conversion with the typical **"as"**
/// keyword. This comes with all benefits and restrictions + possible lossy
/// conversions from it.
pub trait NumFromAs<T> {
    fn from_num(t: T) -> Self;
}

/// Implements the [`NumFromAs`] trait for a given type.
macro_rules! impl_num_from_trait {
    ($source_type: ty, $dest_type: ty) => {
        impl NumFromAs<$source_type> for $dest_type {
            fn from_num(t: $source_type) -> Self {
                t as $dest_type
            }
        }
    };
}

/// Implements the [`NumFromAs`] for all primitive numeric data types
/// to a given type.
macro_rules! impl_num_from_trait_all_to {
    ($float_type: ty) => {
        impl_num_from_trait!(i8, $float_type);
        impl_num_from_trait!(i16, $float_type);
        impl_num_from_trait!(i32, $float_type);
        impl_num_from_trait!(i64, $float_type);
        impl_num_from_trait!(i128, $float_type);
        impl_num_from_trait!(u8, $float_type);
        impl_num_from_trait!(u16, $float_type);
        impl_num_from_trait!(u32, $float_type);
        impl_num_from_trait!(u64, $float_type);
        impl_num_from_trait!(u128, $float_type);
        impl_num_from_trait!(f32, $float_type);
        impl_num_from_trait!(f64, $float_type);
        impl_num_from_trait!(usize, $float_type);
        impl_num_from_trait!(isize, $float_type);
    };
}

// Empowers us to convert all primitive numeric data types of Rust to either f32 or f64,
// by only using type system magic.
impl_num_from_trait_all_to!(f32);
impl_num_from_trait_all_to!(f64);
impl_num_from_trait_all_to!(i8);
impl_num_from_trait_all_to!(i16);
impl_num_from_trait_all_to!(i32);
impl_num_from_trait_all_to!(i64);
impl_num_from_trait_all_to!(i128);
impl_num_from_trait_all_to!(u8);
impl_num_from_trait_all_to!(u16);
impl_num_from_trait_all_to!(u32);
impl_num_from_trait_all_to!(u64);
impl_num_from_trait_all_to!(u128);
impl_num_from_trait_all_to!(usize);
impl_num_from_trait_all_to!(isize);

/// Is to [`NumFromAs`] what [`Into`] is to [`From`].
pub trait NumInto<D> {
    /// Converts the number into the desired numeric type.
    fn into_num(self) -> D;
}

impl<SelfNum, TargetNum> NumInto<TargetNum> for SelfNum
    where
        TargetNum: NumFromAs<SelfNum>,
{
    fn into_num(self) -> TargetNum {
        TargetNum::from_num(self)
    }
}

/// Common super trait for `f32` and `f64`.
pub trait FloatTrait:
Mul<Output = Self> + Div<Output = Self> + Add<Output = Self> + Sub<Output = Self> + Sized + Copy
{
    /// Returns pi.
    fn pi() -> Self;
    /// Returns 1.0. Required because in my generic approach I can't use literals. At least
    /// I didn't got it working.
    fn one() -> Self;
    /// Returns 2.0. Required because in my generic approach I can't use literals. At least
    /// I didn't got it working.
    fn two() -> Self;
}

impl FloatTrait for f32 {
    fn pi() -> Self {
        f32::consts::PI
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }
}
impl FloatTrait for f64 {
    fn pi() -> Self {
        f64::consts::PI
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }
}