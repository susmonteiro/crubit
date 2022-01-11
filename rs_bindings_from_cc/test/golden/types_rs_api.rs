#![rustfmt::skip]
// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#![feature(const_ptr_offset_from, custom_inner_attributes)]

use memoffset_unstable_const::offset_of;

// <unknown location>
// Error while generating bindings for item '__builtin_ms_va_list':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__u_char':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__u_short':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__u_int':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__u_long':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__int8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uint8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__int16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uint16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__int32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uint32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__int64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uint64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__quad_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__u_quad_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__intmax_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uintmax_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__dev_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__uid_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__gid_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__ino_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__ino64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__mode_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__nlink_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__off_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__off64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__pid_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__clock_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__rlim_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__rlim64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__id_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__time_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__useconds_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__suseconds_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__daddr_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__key_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__clockid_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__timer_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__blksize_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__blkcnt_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__blkcnt64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__fsblkcnt_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__fsblkcnt64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__fsfilcnt_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__fsfilcnt64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__fsword_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__ssize_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__syscall_slong_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__syscall_ulong_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__loff_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__caddr_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__intptr_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__socklen_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item '__sig_atomic_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_least8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_least16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_least32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_least64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_least8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_least16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_least32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_least64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_fast8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_fast16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_fast32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'int_fast64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_fast8_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_fast16_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_fast32_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uint_fast64_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'intmax_t':
// Cannot generate bindings for type aliases

// <unknown location>
// Error while generating bindings for item 'uintmax_t':
// Cannot generate bindings for type aliases

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SomeStruct {
    /// Prevent empty C++ struct being zero-size in Rust.
    placeholder: std::mem::MaybeUninit<u8>,
}

// rs_bindings_from_cc/test/golden/types.h;l=7
// Error while generating bindings for item 'SomeStruct::SomeStruct':
// Nested classes are not supported yet

impl Default for SomeStruct {
    #[inline(always)]
    fn default() -> Self {
        let mut tmp = std::mem::MaybeUninit::<Self>::zeroed();
        unsafe {
            crate::detail::__rust_thunk___ZN10SomeStructC1Ev(tmp.as_mut_ptr());
            tmp.assume_init()
        }
    }
}

// rs_bindings_from_cc/test/golden/types.h;l=7
// Error while generating bindings for item 'SomeStruct::SomeStruct':
// Parameter type 'struct SomeStruct &&' is not supported

// rs_bindings_from_cc/test/golden/types.h;l=7
// Error while generating bindings for item 'SomeStruct::operator=':
// Parameter type 'struct SomeStruct &&' is not supported

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FieldTypeTestStruct {
    pub bool_field: bool,
    pub char_field: u8,
    pub unsigned_char_field: u8,
    pub signed_char_field: i8,
    pub char16_t_field: u16,
    pub char32_t_field: u32,
    pub wchar_t_field: i32,
    pub short_field: i16,
    pub int_field: i32,
    pub long_field: i64,
    pub long_long_field: i64,
    pub unsigned_short_field: u16,
    pub unsigned_int_field: u32,
    pub unsigned_long_field: u64,
    pub unsigned_long_long_field: u64,
    pub signed_short_field: i16,
    pub signed_int_field: i32,
    pub signed_long_field: i64,
    pub signed_long_long_field: i64,
    pub int8_t_field: i8,
    pub int16_t_field: i16,
    pub int32_t_field: i32,
    pub int64_t_field: i64,
    pub uint8_t_field: u8,
    pub uint16_t_field: u16,
    pub uint32_t_field: u32,
    pub uint64_t_field: u64,
    pub ptrdiff_t_field: isize,
    pub size_t_field: usize,
    pub intptr_t_field: isize,
    pub uintptr_t_field: usize,
    pub float_field: f32,
    pub double_field: f64,
    pub ptr_field: *mut i32,
    pub struct_field: SomeStruct,
    pub struct_ptr_field: *mut SomeStruct,
    pub const_struct_ptr_field: *const SomeStruct,
    pub struct_ref_field: *mut SomeStruct,
    pub const_struct_ref_field: *const SomeStruct,
}

// rs_bindings_from_cc/test/golden/types.h;l=9
// Error while generating bindings for item 'FieldTypeTestStruct::FieldTypeTestStruct':
// Nested classes are not supported yet

// rs_bindings_from_cc/test/golden/types.h;l=9
// Error while generating bindings for item 'FieldTypeTestStruct::FieldTypeTestStruct':
// Parameter type 'struct FieldTypeTestStruct &&' is not supported

#[inline(always)]
pub fn VoidReturningFunction() {
    unsafe { crate::detail::__rust_thunk___Z21VoidReturningFunctionv() }
}

// CRUBIT_RS_BINDINGS_FROM_CC_TEST_GOLDEN_TYPES_H_

mod detail {
    use super::*;
    extern "C" {
        pub(crate) fn __rust_thunk___ZN10SomeStructC1Ev(__this: *mut SomeStruct);
        pub(crate) fn __rust_thunk___Z21VoidReturningFunctionv();
    }
}

const _: () = assert!(std::mem::size_of::<Option<&i32>>() == std::mem::size_of::<&i32>());

const _: () = assert!(std::mem::size_of::<SomeStruct>() == 1usize);
const _: () = assert!(std::mem::align_of::<SomeStruct>() == 1usize);

const _: () = assert!(std::mem::size_of::<FieldTypeTestStruct>() == 216usize);
const _: () = assert!(std::mem::align_of::<FieldTypeTestStruct>() == 8usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, bool_field) * 8 == 0usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, char_field) * 8 == 8usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, unsigned_char_field) * 8 == 16usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, signed_char_field) * 8 == 24usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, char16_t_field) * 8 == 32usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, char32_t_field) * 8 == 64usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, wchar_t_field) * 8 == 96usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, short_field) * 8 == 128usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, int_field) * 8 == 160usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, long_field) * 8 == 192usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, long_long_field) * 8 == 256usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, unsigned_short_field) * 8 == 320usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, unsigned_int_field) * 8 == 352usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, unsigned_long_field) * 8 == 384usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, unsigned_long_long_field) * 8 == 448usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, signed_short_field) * 8 == 512usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, signed_int_field) * 8 == 544usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, signed_long_field) * 8 == 576usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, signed_long_long_field) * 8 == 640usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, int8_t_field) * 8 == 704usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, int16_t_field) * 8 == 720usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, int32_t_field) * 8 == 736usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, int64_t_field) * 8 == 768usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, uint8_t_field) * 8 == 832usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, uint16_t_field) * 8 == 848usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, uint32_t_field) * 8 == 864usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, uint64_t_field) * 8 == 896usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, ptrdiff_t_field) * 8 == 960usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, size_t_field) * 8 == 1024usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, intptr_t_field) * 8 == 1088usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, uintptr_t_field) * 8 == 1152usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, float_field) * 8 == 1216usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, double_field) * 8 == 1280usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, ptr_field) * 8 == 1344usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, struct_field) * 8 == 1408usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, struct_ptr_field) * 8 == 1472usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, const_struct_ptr_field) * 8 == 1536usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, struct_ref_field) * 8 == 1600usize);
const _: () = assert!(offset_of!(FieldTypeTestStruct, const_struct_ref_field) * 8 == 1664usize);
