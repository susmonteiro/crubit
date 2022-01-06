// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#include <cstddef>
#include <memory>
#include "rs_bindings_from_cc/support/cxx20_backports.h"
#include "rs_bindings_from_cc/test/golden/types.h"

extern "C" void __rust_thunk___ZN10SomeStructC1Ev(SomeStruct* __this) {
  rs_api_impl_support ::construct_at(__this);
}
extern "C" void __rust_thunk___ZN10SomeStructC1ERKS_(
    SomeStruct* __this, const SomeStruct& __param_0) {
  rs_api_impl_support ::construct_at(__this, __param_0);
}
extern "C" void __rust_thunk___ZN10SomeStructD1Ev(SomeStruct* __this) {
  std ::destroy_at(__this);
}
extern "C" void __rust_thunk___ZN19FieldTypeTestStructC1ERKS_(
    FieldTypeTestStruct* __this, const FieldTypeTestStruct& __param_0) {
  rs_api_impl_support ::construct_at(__this, __param_0);
}
extern "C" void __rust_thunk___ZN19FieldTypeTestStructD1Ev(
    FieldTypeTestStruct* __this) {
  std ::destroy_at(__this);
}
extern "C" void __rust_thunk___Z21VoidReturningFunctionv() {
  VoidReturningFunction();
}

static_assert(sizeof(SomeStruct) == 1);
static_assert(alignof(SomeStruct) == 1);

static_assert(sizeof(FieldTypeTestStruct) == 216);
static_assert(alignof(FieldTypeTestStruct) == 8);
static_assert(offsetof(FieldTypeTestStruct, bool_field) * 8 == 0);
static_assert(offsetof(FieldTypeTestStruct, char_field) * 8 == 8);
static_assert(offsetof(FieldTypeTestStruct, unsigned_char_field) * 8 == 16);
static_assert(offsetof(FieldTypeTestStruct, signed_char_field) * 8 == 24);
static_assert(offsetof(FieldTypeTestStruct, char16_t_field) * 8 == 32);
static_assert(offsetof(FieldTypeTestStruct, char32_t_field) * 8 == 64);
static_assert(offsetof(FieldTypeTestStruct, wchar_t_field) * 8 == 96);
static_assert(offsetof(FieldTypeTestStruct, short_field) * 8 == 128);
static_assert(offsetof(FieldTypeTestStruct, int_field) * 8 == 160);
static_assert(offsetof(FieldTypeTestStruct, long_field) * 8 == 192);
static_assert(offsetof(FieldTypeTestStruct, long_long_field) * 8 == 256);
static_assert(offsetof(FieldTypeTestStruct, unsigned_short_field) * 8 == 320);
static_assert(offsetof(FieldTypeTestStruct, unsigned_int_field) * 8 == 352);
static_assert(offsetof(FieldTypeTestStruct, unsigned_long_field) * 8 == 384);
static_assert(offsetof(FieldTypeTestStruct, unsigned_long_long_field) * 8 ==
              448);
static_assert(offsetof(FieldTypeTestStruct, signed_short_field) * 8 == 512);
static_assert(offsetof(FieldTypeTestStruct, signed_int_field) * 8 == 544);
static_assert(offsetof(FieldTypeTestStruct, signed_long_field) * 8 == 576);
static_assert(offsetof(FieldTypeTestStruct, signed_long_long_field) * 8 == 640);
static_assert(offsetof(FieldTypeTestStruct, int8_t_field) * 8 == 704);
static_assert(offsetof(FieldTypeTestStruct, int16_t_field) * 8 == 720);
static_assert(offsetof(FieldTypeTestStruct, int32_t_field) * 8 == 736);
static_assert(offsetof(FieldTypeTestStruct, int64_t_field) * 8 == 768);
static_assert(offsetof(FieldTypeTestStruct, uint8_t_field) * 8 == 832);
static_assert(offsetof(FieldTypeTestStruct, uint16_t_field) * 8 == 848);
static_assert(offsetof(FieldTypeTestStruct, uint32_t_field) * 8 == 864);
static_assert(offsetof(FieldTypeTestStruct, uint64_t_field) * 8 == 896);
static_assert(offsetof(FieldTypeTestStruct, ptrdiff_t_field) * 8 == 960);
static_assert(offsetof(FieldTypeTestStruct, size_t_field) * 8 == 1024);
static_assert(offsetof(FieldTypeTestStruct, intptr_t_field) * 8 == 1088);
static_assert(offsetof(FieldTypeTestStruct, uintptr_t_field) * 8 == 1152);
static_assert(offsetof(FieldTypeTestStruct, float_field) * 8 == 1216);
static_assert(offsetof(FieldTypeTestStruct, double_field) * 8 == 1280);
static_assert(offsetof(FieldTypeTestStruct, ptr_field) * 8 == 1344);
static_assert(offsetof(FieldTypeTestStruct, struct_field) * 8 == 1408);
static_assert(offsetof(FieldTypeTestStruct, struct_ptr_field) * 8 == 1472);
static_assert(offsetof(FieldTypeTestStruct, const_struct_ptr_field) * 8 ==
              1536);
static_assert(offsetof(FieldTypeTestStruct, struct_ref_field) * 8 == 1600);
static_assert(offsetof(FieldTypeTestStruct, const_struct_ref_field) * 8 ==
              1664);
