// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#ifndef CRUBIT_RS_BINDINGS_FROM_CC_IR_FROM_CC_H_
#define CRUBIT_RS_BINDINGS_FROM_CC_IR_FROM_CC_H_

#include <vector>

#include "rs_bindings_from_cc/ir.h"
#include "third_party/absl/strings/string_view.h"
#include "third_party/absl/types/span.h"

namespace rs_bindings_from_cc {

IR IrFromCc(absl::Span<const absl::string_view> header_files_contents,
            const std::vector<absl::string_view>& args);

}  // namespace rs_bindings_from_cc

#endif  // CRUBIT_RS_BINDINGS_FROM_CC_IR_FROM_CC_H_
