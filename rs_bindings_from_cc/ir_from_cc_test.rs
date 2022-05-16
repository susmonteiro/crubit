// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
#![cfg(test)]

use anyhow::Result;
use ir::*;
use ir_testing::*;
use itertools::Itertools;
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use token_stream_matchers::{assert_ir_matches, assert_ir_not_matches, assert_items_match};

#[test]
fn test_function() {
    let ir = ir_from_cc("int f(int a, int b);").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Func {
                name: "f",
                owning_target: BazelLabel("//test:testing_target"),
                mangled_name: "_Z1fii",
                doc_comment: None,
                return_type: MappedType {
                    rs_type: RsType {
                        name: Some("i32"),
                        lifetime_args: [],
                        type_args: [],
                        decl_id: None,
                    },
                    cc_type: CcType {
                        name: Some("int"),
                        is_const: false,
                        type_args: [],
                        decl_id: None,
                    },
                },
                params: [
                    FuncParam {
                        type_: MappedType {
                            rs_type: RsType {
                                name: Some("i32"),
                                lifetime_args: [],
                                type_args: [],
                                decl_id: None,
                            },
                            cc_type: CcType {
                                name: Some("int"),
                                is_const: false,
                                type_args: [],
                                decl_id: None,
                            },
                        },
                        identifier: "a",
                    },
                    FuncParam {
                        type_: MappedType {
                            rs_type: RsType {
                                name: Some("i32"),
                                lifetime_args: [],
                                type_args: [],
                                decl_id: None,
                            },
                            cc_type: CcType {
                                name: Some("int"),
                                is_const: false,
                                type_args: [],
                                decl_id: None,
                            },
                        },
                        identifier: "b",
                    },
                ],
                lifetime_params: [],
                is_inline: false,
                member_func_metadata: None,
                has_c_calling_convention: true,
                is_member_or_descendant_of_class_template: false,
                source_loc: SourceLoc {
                    filename: "ir_from_cc_virtual_header.h",
                    line: 3,
                    column: 1,
                },
                id: ItemId(...),
                enclosing_namespace_id: None,
            }
        }
    );
}

#[test]
fn test_function_with_unnamed_parameters() {
    let ir = ir_from_cc("int f(int, int);").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Func {
                name: "f", ...
                mangled_name: "_Z1fii", ...
                params: [
                    FuncParam {
                        ... identifier: "__param_0", ...
                    },
                    FuncParam {
                        ... identifier: "__param_1", ...
                    },
                ], ...
            }
        }
    );
}

#[test]
fn test_function_with_custom_calling_convention() {
    let ir = ir_from_cc("int f_vectorcall(int, int) [[clang::vectorcall]];").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Func {
                name: "f_vectorcall", ...
                mangled_name: "_Z12f_vectorcallii", ...
                has_c_calling_convention: false, ...
            }
        }
    );
}

#[test]
fn test_functions_from_dependency_are_not_emitted() -> Result<()> {
    let ir = ir_from_cc_dependency("int Add(int a, int b);", "int Multiply(int a, int b);")?;
    assert_ir_matches!(ir, quote! { Func { name: "Add" ... } });
    assert_ir_not_matches!(ir, quote! { Func { name: "Multiply" ... } });
    Ok(())
}

#[test]
fn test_dont_import_record_nested_in_func() {
    let ir = ir_from_cc("inline void f() { struct S{}; }").unwrap();
    assert_ir_not_matches!(ir, quote! { Record { ... "S" ... } });
}

#[test]
fn test_dont_import_unused_class_template_or_specialization() {
    let ir = ir_from_cc("template <class T> struct Template{}; template<> struct Template<int>{};")
        .unwrap();
    assert_ir_not_matches!(ir, quote! { Record { ... "Template" ... } });
}

#[test]
fn test_function_template_not_supported_yet() {
    let ir = ir_from_cc("template<typename SomeParam> void SomeFunctionTemplate() {};").unwrap();
    assert_ir_matches!(
        ir,
        quote! { UnsupportedItem {
          name: "SomeFunctionTemplate",
          message: "Function templates are not supported yet" ...
        }}
    );
}

#[test]
fn test_record_member_variable_access_specifiers() {
    let ir = ir_from_cc(
        "
        struct SomeStruct {
            int default_access_int;
          public:
            int public_int;
          protected:
            int protected_int;
          private:
            int private_int;
        };

        class SomeClass {
          int default_access_int;
        };
    ",
    )
    .unwrap();

    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeStruct", ...
                fields: [
                    Field {
                        identifier: Some("default_access_int") ...
                        access: Public ...
                    },
                    Field {
                        identifier: Some("public_int") ...
                        access: Public ...
                    },
                    Field {
                        identifier: Some("protected_int") ...
                        access: Protected ...
                    },
                    Field {
                        identifier: Some("private_int") ...
                        access: Private ...
                    },
                ] ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeClass", ...
                fields: [
                    Field {
                        identifier: Some("default_access_int") ...
                        access: Private ...
                    }
                ] ...
            }
        }
    );
}

#[test]
fn test_unnamed_fields() {
    let ir = ir_from_cc(
        r#"
        struct WithUnnamedFields {
            int foo;
            int :32;
        };"#,
    )
    .unwrap();

    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "WithUnnamedFields", ...
                fields: [
                    Field { identifier: Some("foo") ... },
                    Field { identifier: None ... },
                ] ...
            }
        }
    );
}

#[test]
fn test_record_private_member_functions_not_present() {
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          public:
            int public_method();
          protected:
            int protected_method();
          private:
            int private_method();
        };
    ",
    )
    .unwrap();

    assert_ir_matches!(ir, quote! { Func { name: "public_method" ... } });
    assert_ir_not_matches!(ir, quote! { Func { name: "protected_method" ... } });
    assert_ir_not_matches!(ir, quote! { Func { name: "private_method" ... } });
}

#[test]
fn test_record_private_static_member_functions_not_present() {
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          public:
            static int public_method();
          protected:
            static int protected_method();
          private:
            static int private_method();
        };
    ",
    )
    .unwrap();

    assert_ir_matches!(ir, quote! { Func { name: "public_method" ... } });
    assert_ir_not_matches!(ir, quote! { Func { name: "protected_method" ... } });
    assert_ir_not_matches!(ir, quote! { Func { name: "private_method" ... } });
}

#[test]
fn test_record_special_member_access_specifiers() {
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          private:
            SomeStruct(SomeStruct& s);
          protected:
            SomeStruct(SomeStruct&& s);
          public:
            ~SomeStruct();
        };
    ",
    )
    .unwrap();

    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeStruct" ...
                copy_constructor: SpecialMemberFunc { ... access: Private ... },
                move_constructor: SpecialMemberFunc { ... access: Protected ... },
                destructor: SpecialMemberFunc { ... access: Public ... } ...
            }
        }
    );
}

#[test]
fn test_record_special_member_definition() {
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          private:
            SomeStruct(SomeStruct& s);
          protected:
            SomeStruct(SomeStruct&& s) = delete;
        };
    ",
    )
    .unwrap();

    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeStruct" ...
                copy_constructor: SpecialMemberFunc { definition: NontrivialUserDefined ... },
                move_constructor: SpecialMemberFunc { definition: Deleted ... },
                destructor: SpecialMemberFunc { definition: Trivial ... } ...
            }
        }
    );
}

#[test]
fn test_pointer_member_variable() {
    let ir = ir_from_cc(
        "struct SomeStruct {
            SomeStruct* ptr;
        };",
    )
    .unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Field {
                identifier: Some("ptr") ...
                type_: MappedType {
                    rs_type: RsType {
                        name: Some("*mut") ...
                        type_args: [RsType {
                            name: None ...
                            type_args: [],
                            decl_id: Some(...),
                        }],
                        decl_id: None,
                    },
                    cc_type: CcType {
                        name: Some("*") ...
                        type_args: [CcType {
                            name: None ...
                            type_args: [],
                            decl_id: Some(...),
                        }],
                        decl_id: None,
                    },
                } ...
            }
        }
    );
}

#[test]
fn test_doc_comment() -> Result<()> {
    let ir = ir_from_cc(
        r#"
            /// Doc comment
            ///
            ///  * with three slashes
            struct DocCommentSlashes {};

            //! Doc comment
            //!
            //!  * with slashes and bang
            struct DocCommentBang {};

            /** Multiline comment

                * with two stars */
            struct MultilineCommentTwoStars {};

            // Line comment
            //
            //  * with two slashes
            struct LineComment {};

            /* Multiline comment

                * with one star */
            struct MultilineOneStar {};
        "#,
    )?;
    let comments: HashMap<_, _> =
        ir.records().map(|r| (r.rs_name.as_str(), r.doc_comment.as_ref().unwrap())).collect();

    assert_eq!(comments["DocCommentSlashes"], "Doc comment\n\n * with three slashes");
    assert_eq!(comments["DocCommentBang"], "Doc comment\n\n * with slashes and bang");

    // TODO(forster): The bullet point is not retained in this
    // case. Instead we get the space at the end. Not sure if this
    // can be fixed easily...
    assert_eq!(comments["MultilineCommentTwoStars"], "Multiline comment\n\n with two stars ");
    assert_eq!(comments["LineComment"], "Line comment\n\n * with two slashes");

    // TODO(forster): The bullet point is not retained in this
    // case. Instead we get the space at the end. Not sure if this
    // can be fixed easily...
    assert_eq!(comments["MultilineOneStar"], "Multiline comment\n\n with one star ");

    Ok(())
}

#[test]
fn test_doc_comment_vs_tooling_directives() -> Result<()> {
    let ir = ir_from_cc(
        r#" // Doc comment for `f1`
            // NOLINTNEXTLINE(google3-readability-pass-trivial-by-value)
            void f1();

            // Doc comment for `f2`
            // // NOLINT
            void f2();

            // // NOLINT
            static void f3();

            // Mid-sentence usage: [...] this is why we need NOLINT / virtual [...].
            void f4();

            // No closing paren still suppresses
            // NOLINTNEXTLINE(google3-readability
            void f5();

            // Multiple, comma-separated directives listed in parens
            // NOLINTNEXTLINE(foo,bar)
            void f6();
        "#,
    )?;

    let comments: HashMap<&str, Option<&str>> = ir
        .functions()
        .map(|f| {
            if let UnqualifiedIdentifier::Identifier(id) = &f.name {
                (id.identifier.as_str(), f.doc_comment.as_deref())
            } else {
                panic!("No constructors/destructors expected in this test.")
            }
        })
        .collect();

    assert_eq!(comments["f1"], Some("Doc comment for `f1`"));
    assert_eq!(comments["f2"], Some("Doc comment for `f2`"));
    assert_eq!(comments["f3"], None);
    assert_eq!(
        comments["f4"],
        Some("Mid-sentence usage: [...] this is why we need NOLINT / virtual [...].")
    );
    assert_eq!(comments["f5"], Some("No closing paren still suppresses"));
    assert_eq!(comments["f6"], Some("Multiple, comma-separated directives listed in parens"));
    Ok(())
}

// TOOD(rosica): Reenable once b/208377928 is fixed. (Also disallow dead code.)
// #[test]
#[allow(dead_code)]
fn test_type_conversion() -> Result<()> {
    // TODO(mboehme): Add tests for the corresponding versions of the types in
    // the `std` namespace. We currently can't do this because we can't include
    // C++ standard library headers such as <cstdint>, only builtin headers such
    // as <stdint.h> (see b/214344126).
    let ir = ir_from_cc(
        r#"
            #include <stdint.h>
            #include <stddef.h>

            // We mock types from the C++ standard library because it's hard to
            // make headers that aren't part of the compiler available to a unit test.
            namespace std {
              using ::int8_t;
              using ::int16_t;
              using ::int32_t;
              using ::int64_t;

              using ::uint8_t;
              using ::uint16_t;
              using ::uint32_t;
              using ::uint64_t;

              using ::ptrdiff_t;
              using ::size_t;
              using ::intptr_t;
              using ::uintptr_t;
            }

            struct S {
                bool b;

                char c;
                unsigned char uc;
                signed char sc;
                char16_t c16;
                char32_t c32;
                wchar_t wc;

                short s;
                int i;
                long l;
                long long ll;

                unsigned short us;
                unsigned int ui;
                unsigned long ul;
                unsigned long long ull;

                signed short ss;
                signed int si;
                signed long sl;
                signed long long sll;

                int8_t i8;
                int16_t i16;
                int32_t i32;
                int64_t i64;
                std::int8_t std_i8;
                std::int16_t std_i16;
                std::int32_t std_i32;
                std::int64_t std_i64;

                uint8_t u8;
                uint16_t u16;
                uint32_t u32;
                uint64_t u64;
                std::uint8_t std_u8;
                std::uint16_t std_u16;
                std::uint32_t std_u32;
                std::uint64_t std_u64;

                ptrdiff_t pt;
                size_t st;
                intptr_t ip;
                uintptr_t up;
                std::ptrdiff_t std_pt;
                std::size_t std_st;
                std::intptr_t std_ip;
                std::uintptr_t std_up;

                float f;
                double d;
            };
        "#,
    )?;
    let fields = ir.records().next().unwrap().fields.iter();
    let type_mapping: HashMap<_, _> = fields
        .map(|f| {
            (
                f.type_.cc_type.name.as_ref().unwrap().as_str(),
                f.type_.rs_type.name.as_ref().unwrap().as_str(),
            )
        })
        .collect();

    assert_eq!(type_mapping["bool"], "bool");

    assert_eq!(type_mapping["char"], "i8");
    assert_eq!(type_mapping["unsigned char"], "u8");
    assert_eq!(type_mapping["signed char"], "i8");
    assert_eq!(type_mapping["char16_t"], "u16");
    // We cannot map C++ char32_t or wchar_t to Rust char,
    // because Rust requires that chars are valid UTF scalar values.
    assert_eq!(type_mapping["char32_t"], "u32");
    assert_eq!(type_mapping["wchar_t"], "i32");

    assert_eq!(type_mapping["short"], "i16");
    assert_eq!(type_mapping["int"], "i32");
    assert_eq!(type_mapping["long"], "i64");
    assert_eq!(type_mapping["long long"], "i64");

    assert_eq!(type_mapping["unsigned short"], "u16");
    assert_eq!(type_mapping["unsigned int"], "u32");
    assert_eq!(type_mapping["unsigned long"], "u64");
    assert_eq!(type_mapping["unsigned long long"], "u64");

    assert_eq!(type_mapping["short"], "i16");
    assert_eq!(type_mapping["int"], "i32");
    assert_eq!(type_mapping["long"], "i64");
    assert_eq!(type_mapping["long long"], "i64");

    assert_eq!(type_mapping["int8_t"], "i8");
    assert_eq!(type_mapping["int16_t"], "i16");
    assert_eq!(type_mapping["int32_t"], "i32");
    assert_eq!(type_mapping["int64_t"], "i64");
    assert_eq!(type_mapping["std::int8_t"], "i8");
    assert_eq!(type_mapping["std::int16_t"], "i16");
    assert_eq!(type_mapping["std::int32_t"], "i32");
    assert_eq!(type_mapping["std::int64_t"], "i64");

    assert_eq!(type_mapping["uint8_t"], "u8");
    assert_eq!(type_mapping["uint16_t"], "u16");
    assert_eq!(type_mapping["uint32_t"], "u32");
    assert_eq!(type_mapping["uint64_t"], "u64");
    assert_eq!(type_mapping["std::uint8_t"], "u8");
    assert_eq!(type_mapping["std::uint16_t"], "u16");
    assert_eq!(type_mapping["std::uint32_t"], "u32");
    assert_eq!(type_mapping["std::uint64_t"], "u64");

    assert_eq!(type_mapping["ptrdiff_t"], "isize");
    assert_eq!(type_mapping["size_t"], "usize");
    assert_eq!(type_mapping["intptr_t"], "isize");
    assert_eq!(type_mapping["uintptr_t"], "usize");
    assert_eq!(type_mapping["std::ptrdiff_t"], "isize");
    assert_eq!(type_mapping["std::size_t"], "usize");
    assert_eq!(type_mapping["std::intptr_t"], "isize");
    assert_eq!(type_mapping["std::uintptr_t"], "usize");

    assert_eq!(type_mapping["float"], "f32");
    assert_eq!(type_mapping["double"], "f64");

    Ok(())
}

#[test]
fn test_typedef() -> Result<()> {
    let ir = ir_from_cc(
        r#"
            // Doc comment for MyTypedefDecl.
            typedef int MyTypedefDecl;

            // Doc comment for MyTypeAliasDecl.
            using MyTypeAliasDecl = int;
        "#,
    )?;

    let int = quote! {
      MappedType {
        rs_type: RsType {
          name: Some("i32"),
          lifetime_args: [],
          type_args: [],
          decl_id: None,
        },
        cc_type: CcType {
          name: Some("int"),
          is_const: false,
          type_args: [],
          decl_id: None,
        },
      }
    };
    assert_ir_matches!(
        ir,
        quote! {
          TypeAlias {
            identifier: "MyTypedefDecl",
            id: ItemId(...),
            owning_target: BazelLabel("//test:testing_target"),
            doc_comment: Some("Doc comment for MyTypedefDecl."),
            underlying_type: #int,
            enclosing_namespace_id: None,
          }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
          TypeAlias {
            identifier: "MyTypeAliasDecl",
            id: ItemId(...),
            owning_target: BazelLabel("//test:testing_target"),
            doc_comment: Some("Doc comment for MyTypeAliasDecl."),
            underlying_type: #int,
            enclosing_namespace_id: None,
          }
        }
    );

    Ok(())
}

#[test]
fn test_typedef_duplicate() -> Result<()> {
    let ir = ir_from_cc(
        r#"
            struct MyStruct {};
            // First doc comment.
            using MyTypeAlias = MyStruct;
            // Second doc comment.
            using MyTypeAlias = MyStruct;
        "#,
    )?;
    // TODO(b/200064504): Figure out if we can (and want to) merge the doc
    // comments from both C++ declarations above. (Currently only the first doc
    // comment makes it through - maybe this is also okay in the long term?)
    assert_ir_matches!(
        ir,
        quote! {
          TypeAlias {
            identifier: "MyTypeAlias",
            ...
            doc_comment: Some("First doc comment."),
            ...
          }
        }
    );
    // Emitting duplicated TypeAliases is undesirable, because Rust disallows
    // redefining a type alias even when the underlying type matches.  See
    // https://play.rust-lang.org/?edition=2021&gist=1c6f79ed41994fa6c89472742ded2f14
    //
    // The implementation avoids duplicated TypeAliases in the following way:
    // 1) LookupDecl gets called with `decl->getCanonicalDecl()`,
    // 2) LookupDecl deduplicates via `lookup_cache_`.
    assert_ir_not_matches!(
        ir,
        quote! {
          TypeAlias {
            identifier: "MyTypeAlias",
            ...
          }
          ...
          TypeAlias {
            identifier: "MyTypeAlias",
            ...
          }
        }
    );
    Ok(())
}

#[test]
fn test_typedef_of_fully_instantiated_template() -> Result<()> {
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision

            // Doc comment of MyStruct template.
            template <typename T>
            struct MyStruct {
              // Doc comment of GetValue method.
              const T& GetValue() const { return value; }

              // Doc comment of `value` field.
              T value;
            };

            // Doc comment of MyTypeAlias.
            using MyTypeAlias = MyStruct<int>; "#,
    )?;
    // Instantiation of the struct template:
    assert_ir_matches!(
        ir,
        quote! {
          Record {
            rs_name: "__CcTemplateInst8MyStructIiE", ...
            cc_name: "MyStruct<int>", ...
            owning_target: BazelLabel("//test:testing_target"), ...
            doc_comment: Some("Doc comment of MyStruct template."), ...
            fields: [Field {
                identifier: Some("value"), ...
                doc_comment: Some("Doc comment of `value` field."), ...
                type_: MappedType {
                    rs_type: RsType { name: Some("i32"), ... },
                    cc_type: CcType { name: Some("int"), ... },
                },
                access: Public,
                offset: 0, ...
            }], ...
          }
        }
    );
    // Make sure the instantiation of the class template appears exactly once in the
    // `top_level_item_ids`.
    let record = ir.records().find(|r| r.cc_name == "MyStruct<int>").unwrap();
    assert_eq!(1, ir.top_level_item_ids().filter(|&&id| id == record.id).count());
    // Type alias for the class template specialization.
    assert_ir_matches!(
        ir,
        quote! {
          TypeAlias {
            identifier: "MyTypeAlias", ...
            owning_target: BazelLabel("//test:testing_target"), ...
            doc_comment: Some("Doc comment of MyTypeAlias."), ...
            underlying_type: MappedType {
                rs_type: RsType {
                    name: None,
                    lifetime_args: [],
                    type_args: [],
                    decl_id: Some(ItemId(...)),
                },
                cc_type: CcType {
                    name: None,
                    is_const: false,
                    type_args: [],
                    decl_id: Some(ItemId(...)),
                },
            } ...
          }
        }
    );
    // Member function of the struct template instantiation:
    assert_ir_matches!(
        ir,
        quote! {
          Func {
            name: "GetValue",
            owning_target: BazelLabel("//test:testing_target"),
            mangled_name: "_ZNK8MyStructIiE8GetValueEv___test_testing_target", ...
            doc_comment: Some("Doc comment of GetValue method."), ...
            is_inline: true, ...
            member_func_metadata: Some(MemberFuncMetadata {
                record_id: ItemId(...),
                instance_method_metadata: Some(InstanceMethodMetadata { ... }), ...
            }), ...
          }
        }
    );
    // Implicitly defined assignment operator inside the struct template is
    // represented in the AST slightly differently (not marked as instantiated)
    // because it is generated by the compiler for the complete, instantiated type
    // according to general rules.
    assert_ir_matches!(
        ir,
        quote! {
          Func {
              name: "operator=",
              owning_target: BazelLabel("//test:testing_target"),
              mangled_name: "_ZN8MyStructIiEaSERKS0____test_testing_target", ...
              doc_comment: None, ...
          }
        }
    );
    Ok(())
}

#[test]
fn test_typedef_for_explicit_template_specialization() -> Result<()> {
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision

            template <typename T>
            struct MyStruct final {};

            // Doc comment for template specialization for T=int.
            template<>
            struct MyStruct<int> final {
              // Doc comment of the GetValue method specialization for T=int.
              const int& GetValue() const { return value * 42; }

              // Doc comment of the `value` field specialization for T=int.
              int value;
            };

            // Doc comment of MyTypeAlias.
            using MyTypeAlias = MyStruct<int>; "#,
    )?;
    // Make sure the explicit specialization of the struct template appears exactly
    // once in the `top_level_item_ids`.
    let record = ir.records().find(|r| r.cc_name == "MyStruct<int>").unwrap();
    assert_eq!(1, ir.top_level_item_ids().filter(|&&id| id == record.id).count());
    // Instantiation of the struct template based on the specialization for T=int:
    assert_ir_matches!(
        ir,
        quote! {
          Record {
            rs_name: "__CcTemplateInst8MyStructIiE", ...
            cc_name: "MyStruct<int>", ...
            owning_target: BazelLabel("//test:testing_target"), ...
            doc_comment: Some("Doc comment for template specialization for T=int."), ...
            fields: [Field {
                identifier: Some("value"), ...
                doc_comment: Some("Doc comment of the `value` field specialization for T=int."), ...
                type_: MappedType {
                    rs_type: RsType { name: Some("i32"), ... },
                    cc_type: CcType { name: Some("int"), ... },
                },
                access: Public,
                offset: 0, ...
            }], ...
          }
        }
    );
    // Instance method inside the struct template:
    assert_ir_matches!(
        ir,
        quote! {
          Func {
            name: "GetValue",
            owning_target: BazelLabel("//test:testing_target"),
            mangled_name: "_ZNK8MyStructIiE8GetValueEv___test_testing_target", ...
            doc_comment: Some("Doc comment of the GetValue method specialization for T=int."), ...
            is_inline: true, ...
            member_func_metadata: Some(MemberFuncMetadata {
                record_id: ItemId(...),
                instance_method_metadata: Some(InstanceMethodMetadata { ... }), ...
            }), ...
          }
        }
    );
    Ok(())
}

#[test]
fn test_multiple_typedefs_to_same_template() -> Result<()> {
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyStruct {
              void MyMethod() {}
            };
            using MyIntAlias = MyStruct<int>;
            using MyIntAlias = MyStruct<int>;
            using MyIntAlias2 = MyStruct<int>;
            using MyFloatAlias = MyStruct<float>;
            "#,
    )?;

    // Verify that there is only 1 record for each specialization.
    assert_eq!(1, ir.records().filter(|r| r.cc_name == "MyStruct<int>").count());
    assert_eq!(1, ir.records().filter(|r| r.cc_name == "MyStruct<float>").count());
    let functions = ir
        .functions()
        .filter(|f| f.name == UnqualifiedIdentifier::Identifier(ir_id("MyMethod")))
        .collect_vec();

    // Verify that there is only 1 function per instantiation.
    assert_eq!(2, functions.len());
    let rec_id1 = functions[0].member_func_metadata.as_ref().unwrap().record_id;
    let rec_id2 = functions[1].member_func_metadata.as_ref().unwrap().record_id;
    assert_ne!(rec_id1, rec_id2);
    Ok(())
}

#[test]
fn test_templates_inheritance() -> Result<()> {
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            class BaseTemplate {
             protected:
              BaseTemplate(T base_value) : base_value_(base_value) {}
              const T& base_value() const { return base_value_; }
             private:
              T base_value_;
            };

            template <typename T>
            class ClassTemplateDerivedFromClassTemplate : public BaseTemplate<T> {
             public:
              ClassTemplateDerivedFromClassTemplate(T base_value, T derived_value)
                  : BaseTemplate<T>(base_value), derived_value_(derived_value) {}
              T combined_value() const {
                return 1000 * BaseTemplate<T>::base_value() + derived_value_;
              }
             private:
              T derived_value_;
            };

            using TypeAliasForClassTemplateDerivedFromClassTemplate =
                    ClassTemplateDerivedFromClassTemplate<int>;
            "#,
    )?;

    // ClassTemplateDerivedFromClassTemplate is instantiated because of
    // TypeAliasForClassTemplateDerivedFromClassTemplate..
    assert_eq!(
        1,
        ir.records()
            .filter(|r| r.cc_name.contains("ClassTemplateDerivedFromClassTemplate"))
            .count()
    );

    // BaseTemplate is *not* instantiated in the generated bindings/IR.  The derived
    // class's bindings work fine without the bindings for the base class (this
    // is also true for non-templated base/derived classes).
    assert_eq!(0, ir.records().filter(|r| r.cc_name.contains("BaseTemplate")).count());
    Ok(())
}

#[test]
fn test_aliased_class_template_instantiated_in_header() -> Result<()> {
    // This aliased class template specialization is instantiated due to the code
    // that is present in the header. We should not corrupt the AST by
    // instantiating again.
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyTemplate {
                const T& GetValue() { return field; }
                T field;
            };

            inline void my_full_instantiation() {
                MyTemplate<int> t;
                t.field = 123;
                t.field = t.GetValue() * 123;
            }

            using MyAlias = MyTemplate<int>; "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! {
          Record {
            rs_name: "__CcTemplateInst10MyTemplateIiE", ...
            cc_name: "MyTemplate<int>", ...
            fields: [Field { identifier: Some("field"), ... }], ...
          }
        }
    );
    assert_ir_matches!(ir, quote! { Func { name: "GetValue", ...  } });
    Ok(())
}

#[test]
fn test_aliased_class_template_partially_instantiated_in_header() -> Result<()> {
    // Similar to `test_aliased_class_template_instantiated_in_header`, but doesn't
    // instantiate all members.
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyTemplate {
                const T& GetValue() { return field; }
                T field;
            };

            inline void my_instantiation() {
                MyTemplate<int> t;
                // Members of MyTemplate are not used/instantiated.
            }

            using MyAlias = MyTemplate<int>; "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! {
          Record {
            rs_name: "__CcTemplateInst10MyTemplateIiE", ...
            cc_name: "MyTemplate<int>", ...
            fields: [Field { identifier: Some("field"), ... }], ...
          }
        }
    );
    assert_ir_matches!(ir, quote! { Func { name: "GetValue", ...  } });
    Ok(())
}

#[test]
fn test_no_instantiation_of_template_only_used_in_private_field() -> Result<()> {
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyTemplate { T field; };

            class MyStruct {
             private:
              MyTemplate<int> private_field_;
            }; "#,
    )?;
    // There should be no instantiated template, just because of the private field.
    // To some extent this test is an early enforcement of the long-term plan for
    // b/226580208 and <internal link>.
    assert_ir_not_matches!(ir, quote! { "field" });
    Ok(())
}

#[test]
fn test_subst_template_type_parm_type_vs_const_when_non_const_template_param() -> Result<()> {
    // This test (and
    // `test_subst_template_type_parm_type_vs_const_when_const_template_param`)
    // verifies that `importer.cc` preserves `const` qualifier attached *both* to
    // QualType associated with:
    // 1) SubstTemplateTypeParm (i.e. the template *argument* has `const`:
    // `MyTemplate<const int>`) 2) TemplateTypeParmType used inside the template
    // definition: `const T& GetConstRef()`
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyTemplate {
                const T& GetConstRef() const { return value; }
                T& GetRef() { return value; }
                T value;
            };

            // Just like the other test_subst_template_type_parm_type_vs_const...
            // test, but using non-*const* int template parameter.
            using MyAlias = MyTemplate<int>; "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! {
            Func {
               name: "GetConstRef", ...
               return_type: MappedType {
                   rs_type: RsType {
                       name: Some("&"), ...
                       type_args: [RsType { name: Some("i32"), ...  }], ...
                   },
                   cc_type: CcType {
                       name: Some("&"),
                       is_const: false,
                       type_args: [CcType {
                           name: Some("int"),
                           is_const: true, ...
                       }], ...
                   },
               }, ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Func {
               name: "GetRef", ...
               return_type: MappedType {
                   rs_type: RsType {
                       name: Some("&mut"), ...
                       type_args: [RsType { name: Some("i32"), ...  }], ...
                   },
                   cc_type: CcType {
                       name: Some("&"),
                       is_const: false,
                       type_args: [CcType {
                           name: Some("int"),
                           is_const: false, ...
                       }], ...
                   },
               }, ...
            }
        }
    );
    Ok(())
}

#[test]
fn test_subst_template_type_parm_type_vs_const_when_const_template_param() -> Result<()> {
    // This test (and
    // `test_subst_template_type_parm_type_vs_const_when_non_const_template_param`)
    // verifies that `importer.cc` preserves `const` qualifier attached *both* to
    // QualType associated with:
    // 1) SubstTemplateTypeParm (i.e. the template *argument* has `const`:
    // `MyTemplate<const int>`) 2) TemplateTypeParmType used inside the template
    // definition: `const T& GetConstRef()`
    let ir = ir_from_cc(
        r#" #pragma clang lifetime_elision
            template <typename T>
            struct MyTemplate {
                const T& GetConstRef() const { return value; }
                T& GetRef() { return value; }
                T value;
            };

            // Just like the other test_subst_template_type_parm_type_vs_const...
            // test, but using *const* int template parameter.
            using MyAlias = MyTemplate<const int>; "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! {
            Func {
               name: "GetConstRef", ...
               return_type: MappedType {
                   rs_type: RsType {
                       name: Some("&"), ...
                       type_args: [RsType { name: Some("i32"), ...  }], ...
                   },
                   cc_type: CcType {
                       name: Some("&"),
                       is_const: false,
                       type_args: [CcType {
                           name: Some("int"),
                           is_const: true, ...
                       }], ...
                   },
               }, ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Func {
               name: "GetRef", ...
               return_type: MappedType {
                   rs_type: RsType {
                       name: Some("&"), ...
                       type_args: [RsType { name: Some("i32"), ...  }], ...
                   },
                   cc_type: CcType {
                       name: Some("&"),
                       is_const: false,
                       type_args: [CcType {
                           name: Some("int"),
                           is_const: true, ...
                       }], ...
                   },
               }, ...
            }
        }
    );
    Ok(())
}

#[test]
fn test_template_and_alias_are_both_in_dependency() -> Result<()> {
    // See also the `test_template_in_dependency_and_alias_in_current_target` test.
    let ir = {
        let dependency_src = r#" #pragma clang lifetime_elision
                template <typename T>
                struct MyTemplate {
                    T GetValue();
                    T field;
                };
                using MyAliasOfTemplate = MyTemplate<int>;
                struct StructInDependency {}; "#;
        let current_target_src = r#" #pragma clang lifetime_elision
                /* no references to MyTemplate or MyAliasOfTemplate */
                struct StructInCurrentTarget {}; "#;
        ir_from_cc_dependency(current_target_src, dependency_src)?
    };

    // Just double-checking the test inputs VS target names.
    let current_target = ir_testing::TESTING_TARGET;
    let dependency = ir_testing::DEPENDENCY_TARGET;
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "StructInCurrentTarget", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "StructInDependency", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );

    // Type alias is only defined in `dependency`.
    assert_ir_matches!(
        ir,
        quote! {
            TypeAlias { ...
                identifier: "MyAliasOfTemplate", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    assert_ir_not_matches!(
        ir,
        quote! {
            TypeAlias { ...
                identifier: "MyAliasOfTemplate", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    // The template should be instantiated in `dependency`, rather than in
    // `current_target`.
    // TODO(b/222001243): Fix which target contains the instantiations and then flip
    // the test assertions below.  Tentative fix: cl/438580040.
    assert_ir_not_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "MyTemplate<int>", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "MyTemplate<int>", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    // The template instantiations in the `dependency` should only produce type
    // information (e.g. TypeAlias, Record) and don't need to produce Func
    // items.
    assert_ir_not_matches!(
        ir,
        quote! {
            Func { ...
                name: "GetValue", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    // There should be nothing template-instantiation-related in the main test
    // target. TODO(b/222001243): Fix which target contains the instantiations
    // and then flip the test assertions below to `assert_ir_not_matches`.
    assert_ir_matches!(
        ir,
        quote! {
            Func { ...
                name: "GetValue", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    Ok(())
}

#[test]
fn test_template_in_dependency_and_alias_in_current_target() -> Result<()> {
    // See also the `test_template_and_alias_are_both_in_dependency` test.
    let ir = {
        let dependency_src = r#" #pragma clang lifetime_elision
                template <typename T>
                struct MyTemplate {
                    T GetValue();
                    T field;
                };
                struct StructInDependency{}; "#;
        let current_target_src = r#" #pragma clang lifetime_elision
                using MyAliasOfTemplate = MyTemplate<int>;
                struct StructInCurrentTarget{}; "#;
        ir_from_cc_dependency(current_target_src, dependency_src)?
    };

    // Just double-checking the test inputs VS target names.
    let current_target = ir_testing::TESTING_TARGET;
    let dependency = ir_testing::DEPENDENCY_TARGET;
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "StructInCurrentTarget", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "StructInDependency", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );

    // Type alias is only defined in `current_target`
    assert_ir_not_matches!(
        ir,
        quote! {
            TypeAlias { ...
                identifier: "MyAliasOfTemplate", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            TypeAlias { ...
                identifier: "MyAliasOfTemplate", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    // The template should be instantiated in `current_target`, rather than in
    // `dependency`.
    assert_ir_not_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "MyTemplate<int>", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    assert_ir_matches!(
        ir,
        quote! {
            Record { ...
                cc_name: "MyTemplate<int>", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    // There should be nothing template-instantiation-related in the dependency
    // (since there is no instantiation there).
    assert_ir_not_matches!(
        ir,
        quote! {
            Func { ...
                name: "GetValue", ...
                owning_target: BazelLabel(#dependency), ...
            }
        }
    );
    // The template instantiations in the current target should produce not only
    // type information (e.g. TypeAlias, Record) but also Func items (for
    // methods of the instantiated class template).
    assert_ir_matches!(
        ir,
        quote! {
            Func { ...
                name: "GetValue", ...
                owning_target: BazelLabel(#current_target), ...
            }
        }
    );

    Ok(())
}

#[test]
fn test_well_known_types_check_namespaces() -> Result<()> {
    // Check that we don't treat a type called `int32_t` in a user-defined
    // namespace as if it was the standard type `int32_t`.
    // Because we don't support namespaces yet, the outcome of this should be
    // that `f()` is unsupported, rather than being imported with a parameter
    // type of `i32`.
    // Once we support namespaces, change this test to check that `f()` is
    // imported with the correct paramter type `my_namespace::int32_t`.
    let ir = ir_from_cc(
        r#"
            namespace my_namespace {
              using int32_t = int;
            }
            void f(my_namespace::int32_t i);
        "#,
    )?;
    assert_strings_contain(
        ir.unsupported_items().map(|i| i.name.as_str()).collect_vec().as_slice(),
        "f",
    );
    Ok(())
}

#[test]
fn test_dont_import_typedef_nested_in_func() {
    let ir = ir_from_cc("inline void f() { typedef int MyTypedefDecl; }").unwrap();
    assert_ir_not_matches!(ir, quote! { TypeAlias { identifier: "MyTypedefDecl" ... } });
}

#[test]
fn test_typedef_nested_in_record_not_supported() {
    let ir = ir_from_cc("struct S { typedef int MyTypedefDecl; };").unwrap();
    assert_strings_contain(
        ir.unsupported_items().map(|i| i.name.as_str()).collect_vec().as_slice(),
        "S::MyTypedefDecl",
    );
}

#[test]
fn test_records_nested_in_records_not_supported_yet() {
    let ir = ir_from_cc("struct SomeStruct { struct NestedStruct {}; };").unwrap();
    assert_ir_matches!(
        ir,
        quote! { UnsupportedItem {
          name: "SomeStruct::NestedStruct",
          message: "Nested classes are not supported yet" ...
        }}
    );
}

#[test]
fn test_record_with_unsupported_field() -> Result<()> {
    // Using a nested struct because it's currently not supported.
    // But... any other unsupported type would also work for this test.
    let ir = ir_from_cc(
        r#"
        struct StructWithUnsupportedField {
          struct NestedStruct {};
          NestedStruct my_field;
        };
    "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! {
              UnsupportedItem(UnsupportedItem {
                name: "StructWithUnsupportedField",
                message: "UNIMPLEMENTED: Type of field 'my_field' is not supported: Unsupported type 'struct StructWithUnsupportedField::NestedStruct': No generated bindings found for 'NestedStruct'",
                ...
            })
        }
    );
    Ok(())
}

#[test]
fn test_record_with_unsupported_base() -> Result<()> {
    let ir = ir_from_cc(
        r#" struct OuterStruct {
              struct NestedStruct {
                // Having a field here avoids empty base class optimization
                // and forces `derived_field` to be at a non-zero offset.
                // See also: https://en.cppreference.com/w/cpp/language/ebo
                char nested_field;
              };
            };

            // Using a nested struct as a base class because nested structs are
            // currently unsupported.  But... any other unsupported base class
            // would also work for this test.
            struct DerivedClass : public OuterStruct::NestedStruct {
              int derived_field;
            }; "#,
    )?;
    // Verify that `unambiguous_public_bases` are empty (instead of containing a
    // dangling `ItemId` of the `NestedStruct` (which got imported as
    // `UnsupportedItem` rather than as a `Record`).
    assert_ir_matches!(
        ir,
        quote! {
           Record {
               rs_name: "DerivedClass",
               cc_name: "DerivedClass",
               id: ItemId(...),
               owning_target: BazelLabel("//test:testing_target"),
               doc_comment: Some(...),
               unambiguous_public_bases: [],
               fields: [Field {
                   identifier: Some("derived_field"), ...
                   offset: 32, ...
               }], ...
               size: 8,
               alignment: 4,
               is_derived_class: true,
               override_alignment: true,
               ...
           }
        }
    );
    // Verify that the NestedStruct is unsupported (this is mostly verification
    // that the test input correctly sets up the test scenario;  the real
    // verification is above).
    assert_ir_matches!(
        ir,
        quote! {
           UnsupportedItem {
               name: "OuterStruct::NestedStruct",
               message: "Nested classes are not supported yet",
               ...
           }
        }
    );
    Ok(())
}

#[test]
fn test_do_not_import_static_member_functions_when_record_not_supported_yet() {
    // only using nested struct as an example of a record we cannot import yet.
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          struct NestedStruct {
            static void StaticMemberFunction();
          };
        };",
    )
    .unwrap();
    assert_ir_matches!(
        ir,
        quote! { UnsupportedItem {
          name: "SomeStruct::NestedStruct::StaticMemberFunction" ...
        }}
    );
}

#[test]
fn test_do_not_import_nonstatic_member_functions_when_record_not_supported_yet() {
    // only using nested struct as an example of a record we cannot import yet.
    let ir = ir_from_cc(
        "
        struct SomeStruct {
          struct NestedStruct {
            void NonStaticMemberFunction();
          };
        };",
    )
    .unwrap();
    assert_ir_matches!(
        ir,
        quote! { UnsupportedItem {
          name: "SomeStruct::NestedStruct::NonStaticMemberFunction" ...
        }}
    );
}

#[test]
fn test_dont_import_injected_class_name() {
    let ir = ir_from_cc("struct SomeStruct {};").unwrap();
    let names = ir.records().map(|r| &r.rs_name).filter(|n| n.contains("SomeStruct"));
    // if we do support nested structs, we should not emit record for injected class
    // name
    assert_eq!(names.count(), 1);
    // if we don't support nested structs, we should not emit unsupported item for
    // injected class name
    assert_ir_not_matches!(
        ir,
        quote! { UnsupportedItem {
          name: "SomeStruct::SomeStruct",
          message: "Nested classes are not supported yet" ...
        }}
    );
}

#[test]
fn test_integer_typedef_usage() -> Result<()> {
    // This is a regression test. We used to incorrectly desugar typedefs of
    // builtin types and treat them as if they were the underlying builtin type.
    // As a result, this test would produce a binding for f(MyTypedef) with a
    // parameter of type `int` instead of `MyTypedef`. This test therefore
    // checks that the type has a `decl_id` but doesn't have a `name`. More
    // specific checks are done in the code generation tests.
    let ir = ir_from_cc(
        r#"
            typedef int MyTypedef;
            void f(MyTypedef my_typedef);
        "#,
    )?;
    assert_ir_matches!(
        ir,
        quote! { Func {
         name: "f", ...
         params: [
           FuncParam {
             type_: MappedType {
               rs_type: RsType {
                 name: None, ...
                 decl_id: Some(...), ...
               },
               cc_type: CcType {
                 name: None, ...
                 decl_id: Some(...), ...
               },
             },
             identifier: "my_typedef",
           }], ...
        } }
    );

    Ok(())
}

#[test]
fn test_struct() {
    let ir = ir_from_cc("struct SomeStruct { int first_field; int second_field; };").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeStruct" ...
                cc_name: "SomeStruct" ...
                fields: [
                    Field {
                        identifier: Some("first_field"), ...
                        type_ : MappedType {
                            rs_type : RsType { name : Some ("i32"), ...},
                            cc_type : CcType { name : Some ("int"), ...},
                         }, ...
                        offset: 0, ...
                    },
                    Field {
                        identifier: Some("second_field"), ...
                        type_ : MappedType {
                            rs_type : RsType { name : Some ("i32"), ...},
                            cc_type : CcType { name : Some ("int"), ...},
                         }, ...
                        offset: 32, ...
                    },
                ], ...
                size: 8, ...
                alignment: 4, ...
                is_union: false, ...
            }
        }
    );
}

#[test]
fn test_struct_forward_declaration() {
    let ir = ir_from_cc("struct Struct;").unwrap();
    assert!(!ir.records().any(|r| r.rs_name == "Struct"));
}

#[test]
fn test_union() {
    let ir = ir_from_cc("union SomeUnion { int first_field; int second_field; };").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            Record {
                rs_name: "SomeUnion" ...
                cc_name: "SomeUnion" ...
                fields: [
                    Field {
                        identifier: Some("first_field"), ...
                        type_ : MappedType {
                            rs_type : RsType { name : Some ("i32"), ...},
                            cc_type : CcType { name : Some ("int"), ...},
                         }, ...
                        offset: 0, ...
                    },
                    Field {
                        identifier: Some("second_field"), ...
                        type_ : MappedType {
                            rs_type : RsType { name : Some ("i32"), ...},
                            cc_type : CcType { name : Some ("int"), ...},
                         }, ...
                        offset: 0, ...
                    },
                ], ...
                size: 4, ...
                alignment: 4, ...
                is_union: true, ...
            }
        }
    );
}

#[test]
fn test_member_function_params() {
    let ir = ir_from_cc(
        r#"
            struct Struct {
                void Foo(int x, int y);
            };
        "#,
    )
    .unwrap();
    let foo_func =
        ir.functions().find(|f| f.name == UnqualifiedIdentifier::Identifier(ir_id("Foo"))).unwrap();
    let param_names: Vec<_> = foo_func.params.iter().map(|p| &p.identifier.identifier).collect();
    assert_eq!(param_names, vec!["__this", "x", "y"]);
}

fn assert_member_function_with_predicate_has_instance_method_metadata<F: FnMut(&Func) -> bool>(
    ir: &IR,
    record_name: &str,
    mut func_predicate: F,
    expected_metadata: &Option<ir::InstanceMethodMetadata>,
) {
    let record = ir.records().find(|r| r.rs_name == record_name).expect("Struct not found");
    let function = ir.functions().find(|f| func_predicate(*f));
    let meta = function
        .expect("Function not found")
        .member_func_metadata
        .as_ref()
        .expect("Member function should specify member_func_metadata");
    assert_eq!(meta.record_id, record.id);
    assert_eq!(&meta.instance_method_metadata, expected_metadata);
}

fn assert_member_function_has_instance_method_metadata(
    name: &str,
    definition: &str,
    expected_metadata: &Option<ir::InstanceMethodMetadata>,
) {
    let mut file = String::new();
    file += "struct Struct {\n  ";
    file += definition;
    file += "\n};";
    let ir = ir_from_cc(&file).unwrap();

    assert_member_function_with_predicate_has_instance_method_metadata(
        &ir,
        "Struct",
        |f| f.name == UnqualifiedIdentifier::Identifier(ir_id(name)),
        expected_metadata,
    );
}

#[test]
fn test_member_function_static() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "static void Function();",
        &None,
    );
}

#[test]
fn test_member_function() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "void Function();",
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::Unqualified,
            is_const: false,
            is_virtual: false,
            is_explicit_ctor: false,
        }),
    );
}

#[test]
fn test_member_function_const() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "void Function() const;",
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::Unqualified,
            is_const: true,
            is_virtual: false,
            is_explicit_ctor: false,
        }),
    );
}

#[test]
fn test_member_function_virtual() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "virtual void Function();",
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::Unqualified,
            is_const: false,
            is_virtual: true,
            is_explicit_ctor: false,
        }),
    );
}

#[test]
fn test_member_function_lvalue() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "void Function() &;",
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::LValue,
            is_const: false,
            is_virtual: false,
            is_explicit_ctor: false,
        }),
    );
}

#[test]
fn test_member_function_rvalue() {
    assert_member_function_has_instance_method_metadata(
        "Function",
        "void Function() &&;",
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::RValue,
            is_const: false,
            is_virtual: false,
            is_explicit_ctor: false,
        }),
    );
}

#[test]
fn test_member_function_explicit_constructor() {
    let ir = ir_from_cc(
        r#"
        struct SomeStruct {
          explicit SomeStruct(int i);
          SomeStruct() = delete;
          SomeStruct(const SomeStruct&) = delete;
        }; "#,
    )
    .unwrap();
    assert_member_function_with_predicate_has_instance_method_metadata(
        &ir,
        "SomeStruct",
        |f| f.name == UnqualifiedIdentifier::Constructor,
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::Unqualified,
            is_const: false,
            is_virtual: false,
            is_explicit_ctor: true,
        }),
    );
}

#[test]
fn test_member_function_implicit_constructor() {
    let ir = ir_from_cc(
        r#"
        struct SomeStruct {
          SomeStruct(int i);
          SomeStruct() = delete;
          SomeStruct(const SomeStruct&) = delete;
        }; "#,
    )
    .unwrap();
    assert_member_function_with_predicate_has_instance_method_metadata(
        &ir,
        "SomeStruct",
        |f| f.name == UnqualifiedIdentifier::Constructor,
        &Some(ir::InstanceMethodMetadata {
            reference: ir::ReferenceQualification::Unqualified,
            is_const: false,
            is_virtual: false,
            is_explicit_ctor: false,
        }),
    );
}

fn get_func_names(definition: &str) -> Vec<ir::UnqualifiedIdentifier> {
    let ir = ir_from_cc(definition).unwrap();
    ir.functions().map(|f| f.name.clone()).collect()
}

#[test]
fn test_identifier_function_name() {
    assert_eq!(
        get_func_names("void Function();"),
        vec![ir::UnqualifiedIdentifier::Identifier(ir::Identifier {
            identifier: "Function".into()
        })],
    );
}

#[test]
fn test_constructor_function_name() {
    assert!(
        get_func_names("struct Struct {Struct();};")
            .contains(&ir::UnqualifiedIdentifier::Constructor)
    );
}

#[test]
fn test_destructor_function_name() {
    assert!(
        get_func_names("struct Struct {~Struct();};")
            .contains(&ir::UnqualifiedIdentifier::Destructor)
    );
}

#[test]
fn test_unsupported_items_are_emitted() -> Result<()> {
    // We will have to rewrite this test to use something else that is unsupported
    // once we start importing nested structs.
    let ir = ir_from_cc("struct X { struct Y {}; };")?;
    assert_strings_contain(
        ir.unsupported_items().map(|i| i.name.as_str()).collect_vec().as_slice(),
        "X::Y",
    );
    Ok(())
}

#[test]
fn test_unsupported_items_from_dependency_are_not_emitted() -> Result<()> {
    // We will have to rewrite this test to use something else that is unsupported
    // once we start importing nested structs.
    let ir = ir_from_cc_dependency(
        "struct MyOtherStruct { OuterStruct::NestedStructIsUnsupported my_field; };",
        "struct OuterStruct { struct NestedStructIsUnsupported {}; };",
    )?;
    let names = ir.unsupported_items().map(|i| i.name.as_str()).collect_vec();
    assert_strings_dont_contain(names.as_slice(), "OuterStruct");
    assert_strings_dont_contain(names.as_slice(), "NestedStructIsUnsupported");
    assert_strings_contain(names.as_slice(), "MyOtherStruct");
    Ok(())
}

#[test]
fn test_user_of_unsupported_type_is_unsupported() -> Result<()> {
    // We will have to rewrite this test to use something else that is unsupported
    // once we start importing nested structs.
    let ir = ir_from_cc(
        r#"struct S { struct Nested {int x;}; int y; };
           void f(S::Nested n);
        "#,
    )?;
    let names = ir.unsupported_items().map(|i| i.name.as_str()).collect_vec();
    assert_strings_contain(&names, "S::Nested");
    assert_strings_contain(&names, "f");
    Ok(())
}

fn assert_strings_contain(strings: &[&str], expected_string: &str) {
    assert!(
        strings.iter().any(|s| *s == expected_string),
        "Value '{}' was unexpectedly missing from {:?}",
        expected_string,
        strings
    );
}

fn assert_strings_dont_contain(strings: &[&str], unexpected_pattern: &str) {
    assert!(
        strings.iter().all(|s| !s.contains(unexpected_pattern)),
        "Pattern {:?} was unexpectedly found in {:?}",
        unexpected_pattern,
        strings
    );
}

#[test]
fn test_elided_lifetimes() {
    let ir = ir_from_cc(
        r#"#pragma clang lifetime_elision
        struct S {
          int& f(int& i);
        };"#,
    )
    .unwrap();
    let func = retrieve_func(&ir, "f");
    let lifetime_params = &func.lifetime_params;
    assert_eq!(lifetime_params.iter().map(|p| &p.name).collect_vec(), vec!["a", "b"]);
    let a_id = lifetime_params[0].id;
    let b_id = lifetime_params[1].id;
    assert_eq!(func.return_type.rs_type.lifetime_args, vec![a_id]);

    assert_eq!(func.params[0].identifier, ir_id("__this"));
    assert_eq!(func.params[0].type_.rs_type.name, Some("&mut".to_string()));
    assert_eq!(func.params[0].type_.rs_type.lifetime_args, vec![a_id]);

    assert_eq!(func.params[1].identifier, ir_id("i"));
    assert_eq!(func.params[1].type_.rs_type.name, Some("&mut".to_string()));
    assert_eq!(func.params[1].type_.rs_type.lifetime_args, vec![b_id]);
}

fn verify_elided_lifetimes_in_default_constructor(ir: &IR) {
    let r = ir.records().next().expect("IR should contain `struct S`");
    assert_eq!(r.rs_name, "S");
    assert!(r.is_trivial_abi);

    let f = ir
        .functions()
        .find(|f| matches!(&f.name, UnqualifiedIdentifier::Constructor) && f.params.len() == 1)
        .expect("IR should contain the default constructor");
    assert_eq!(f.lifetime_params.len(), 1);

    let p = f.params.first().expect("IR should contain `__this` parameter");
    assert_eq!(p.identifier, ir_id("__this"));

    let t = &p.type_.rs_type;
    assert_eq!(t.lifetime_args.len(), 1);
    assert_eq!(t.lifetime_args[0], f.lifetime_params[0].id);
    assert_eq!(t.name, Some("&mut".to_string()));
}

#[test]
fn test_operator_names() {
    let ir = ir_from_cc(
        r#"
        // TOOD(b/208377928): Use #include <stddef.h> instead of declaring `size_t` ourselves...
        using size_t = unsigned long;
        #pragma clang lifetime_elision
        struct SomeStruct {
          // There is an implicit/default `oparator=` hidden here as well.
          void* operator new(size_t size);
          void* operator new[](size_t size);
          bool operator==(const SomeStruct& other) const;
        };"#,
    )
    .unwrap();
    let operator_names: HashSet<&str> = ir
        .functions()
        .filter(|f| {
            // Only SomeStruct member functions (excluding stddef.h stuff).
            f.member_func_metadata
                .as_ref()
                .map(|m| m.find_record(&ir).unwrap().rs_name == "SomeStruct")
                .unwrap_or_default()
        })
        .flat_map(|f| match &f.name {
            UnqualifiedIdentifier::Operator(op) => Some(op.name.as_ref()),
            _ => None,
        })
        .collect();
    assert!(operator_names.contains("="));
    assert!(operator_names.contains("new"));
    assert!(operator_names.contains("new[]"));
    assert!(operator_names.contains("=="));
}

#[test]
fn test_elided_lifetimes_in_default_constructor_with_implicit_default() {
    let ir = ir_from_cc(
        r#"#pragma clang lifetime_elision
        struct S {
          int i;
        };"#,
    )
    .unwrap();
    verify_elided_lifetimes_in_default_constructor(&ir);
}

#[test]
fn test_elided_lifetimes_in_default_constructor_with_explicit_default() {
    let ir = ir_from_cc(
        r#"#pragma clang lifetime_elision
        struct S {
          S() = default;
          int i;
        };"#,
    )
    .unwrap();
    verify_elided_lifetimes_in_default_constructor(&ir);
}

#[test]
fn test_no_aligned_attr() {
    let ir = ir_from_cc("struct SomeStruct {};").unwrap();

    assert_ir_matches! {ir, quote! {
      Record {
        ... rs_name: "SomeStruct" ...
        ... override_alignment: false ...
      }}
    };
}

#[test]
fn test_aligned_attr() {
    let ir = ir_from_cc("struct SomeStruct {} __attribute__((aligned(64)));").unwrap();

    assert_ir_matches! {ir, quote! {
      Record {
        ... rs_name: "SomeStruct" ...
        ... override_alignment: true ...
      }}
    };
}

#[test]
fn test_volatile_is_unsupported() {
    let ir = ir_from_cc("volatile int* foo();").unwrap();
    let f = ir.unsupported_items().find(|i| i.message.contains("volatile")).unwrap();
    assert_eq!("foo", f.name);
}

#[test]
fn test_unnamed_enum_unsupported() {
    let ir = ir_from_cc("enum { kFoo = 1, kBar = 2 };").unwrap();
    assert_ir_matches!(
        ir,
        quote! {
            UnsupportedItem {
                name: "(anonymous)",
                message: "Unnamed enums are not supported yet" ...
            }
        }
    );
}

#[test]
fn test_unsupported_item_has_item_id() {
    let ir = ir_from_cc("struct SomeStruct { struct NestedStruct {}; };").unwrap();
    let unsupported =
        ir.unsupported_items().find(|i| i.name == "SomeStruct::NestedStruct").unwrap();
    assert_ne!(unsupported.id, ItemId(0));
}

#[test]
fn test_comment_has_item_id() {
    let ir = ir_from_cc("// Comment").unwrap();
    let comment = ir.comments().find(|i| i.text == "Comment").unwrap();
    assert_ne!(comment.id, ItemId(0));
}

#[test]
fn test_function_has_item_id() {
    let ir = ir_from_cc("int foo();").unwrap();
    let function =
        ir.functions().find(|i| i.name == UnqualifiedIdentifier::Identifier(ir_id("foo"))).unwrap();
    assert_ne!(function.id, ItemId(0));
}

#[test]
fn test_top_level_items() {
    let ir = ir_from_cc(
        r#"
        struct ForwardDeclaredTopLevelStruct;
        struct TopLevelStruct;
        struct TopLevelStruct {};
        // Top level comment

        // Function comment
        void top_level_func();
        namespace top_level_namespace {
        struct Nested {};
        // free nested comment

        // nested_func comment
        void nested_func();
        }  // namespace top_level_namespace"#,
    )
    .unwrap();

    let top_level_items =
        ir.top_level_item_ids().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_items_match!(
        top_level_items,
        vec![
            quote! {
              IncompleteRecord {
                ... cc_name: "ForwardDeclaredTopLevelStruct" ...
              }
            },
            quote! {
              Record {
                ... rs_name: "TopLevelStruct" ...
              }
            },
            quote! {
              Comment {
                ... text: "Top level comment" ...
              }
            },
            quote! {
              Func { ... name: "top_level_func" ... }
            },
            quote! {
              UnsupportedItem { ... name: "top_level_namespace" ... }
            },
            quote! {
              Comment {
                ... text: "namespace top_level_namespace" ...
              }
            },
        ]
    );
}

#[test]
fn test_record_items() {
    let ir = ir_from_cc(
        r#"
        struct TopLevelStruct {
          // A free comment

          // foo comment
          int foo;

          int bar();
          struct Nested {};
          int baz();
        };"#,
    )
    .unwrap();

    let record = ir.records().find(|i| i.rs_name.as_str() == "TopLevelStruct").unwrap();
    let record_items =
        record.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_items_match!(
        record_items,
        vec![
            quote! {
              Func { ... name: Constructor ... }
            },
            quote! {
              Func { ... name: Constructor ... }
            },
            quote! {
              // Unsupported parameter
              UnsupportedItem { ... name: "TopLevelStruct::TopLevelStruct", ... }
            },
            quote! {
              Func { ... name: Destructor ... }
            },
            quote! {
              Func { ... name: "operator=" ... }
            },
            quote! {
              // Unsupported parameter
              UnsupportedItem { ... name: "TopLevelStruct::operator=" ... }
            },
            quote! {
              ...Comment {
                ... text: "A free comment" ...
              }
            },
            quote! {
              ... Func { ... name: "bar" ... }
            },
            quote! {
              ... UnsupportedItem { ... name: "TopLevelStruct::Nested" ... }
            },
            quote! {
              ...Func {
                ... name: "baz" ...
              }
            },
        ]
    );
}

#[test]
fn test_namespaces() {
    let ir = ir_from_cc(
        r#"
        namespace test_namespace_bindings {
          // A free comment

          // Struct comment
          struct StructWithinNamespace {};

          void function_within_namespace();

          namespace inner_namespace {
          struct InnerStruct {};
          }  // namespace inner_namespace
          }  // namespace test_namespace_bindings"#,
    )
    .unwrap();

    let namespace = ir.namespaces().find(|n| n.name == ir_id("test_namespace_bindings")).unwrap();
    let namespace_items =
        namespace.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_ir_matches!(
        ir,
        quote! {
            ...
            Namespace {
                name: "test_namespace_bindings" ...
                id: ItemId(...) ...
                canonical_namespace_id: ItemId(...) ...
                owning_target: BazelLabel("//test:testing_target") ...
            }
            ...
        }
    );

    assert_items_match!(
        namespace_items,
        vec![
            quote! {
              Comment {
                ... text: "A free comment" ...
              }
            },
            quote! {
              Record {
                ... rs_name : "StructWithinNamespace" ...
              }
            },
            quote! {
              Func { ... name: "function_within_namespace" ... }
            },
            quote! {
              Namespace { ... name: "inner_namespace" ... }
            },
            quote! {
              Comment {
                ... text: "namespace inner_namespace" ...
              }
            },
        ]
    );
}

#[test]
fn test_nested_namespace_definition() {
    let ir = ir_from_cc(
        r#"
        namespace test_namespace_bindings::inner {
        void func();
        }"#,
    )
    .unwrap();

    let namespace = ir.namespaces().find(|n| n.name == ir_id("test_namespace_bindings")).unwrap();
    let namespace_items =
        namespace.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_items_match!(
        namespace_items,
        vec![quote! {
          Namespace { ... name: "inner" ... }
        },]
    );

    let inner_namespace = ir.namespaces().find(|n| n.name == ir_id("inner")).unwrap();
    let inner_namespace_items =
        inner_namespace.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_items_match!(
        inner_namespace_items,
        vec![quote! {
          Func { ... name: "func" ... }
        },]
    );
}

#[test]
fn test_enclosing_namespace_ids() {
    let ir = ir_from_cc(
        r#"
        namespace test_namespace_bindings {
          struct T {};
          struct S {
            void processT();
          };
          void f();
          enum E {};
          typedef int TypedefDecl;
          using TypeAliasDecl = int;
          namespace inner {
            struct InnerS {};
            void inner_f();
            enum InnerE {};
            typedef int InnerTypedefDecl;
            using InnerTypeAliasDecl = int;
          }
        }"#,
    )
    .unwrap();

    let namespace = ir.namespaces().find(|n| n.name == ir_id("test_namespace_bindings")).unwrap();
    let namespace_items: Vec<&Item> =
        namespace.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert_eq!(namespace.enclosing_namespace_id, None);
    assert!(namespace_items.iter().all(|item| item.enclosing_namespace_id() == Some(namespace.id)));

    let inner_namespace = ir.namespaces().find(|n| n.name == ir_id("inner")).unwrap();
    let inner_namespace_items: Vec<&Item> =
        inner_namespace.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();

    assert!(
        inner_namespace_items
            .iter()
            .all(|item| item.enclosing_namespace_id() == Some(inner_namespace.id))
    );

    let record = ir.records().find(|r| r.rs_name.as_str() == "S").unwrap();
    let record_items: Vec<&Item> =
        record.child_item_ids.iter().map(|id| ir.find_decl(*id).unwrap()).collect_vec();
    for item in record_items.iter() {
        match item {
            Item::UnsupportedItem(_) => {}
            Item::Comment(_) => {}
            _ => {
                assert!(item.enclosing_namespace_id() == Some(namespace.id));
            }
        }
    }
}

#[test]
fn test_namespace_canonical_id() {
    let ir = ir_from_cc(
        r#"
        namespace test_namespace_bindings {
          struct T {};
        }
        int i;
        namespace test_namespace_bindings {
          struct Y {};
        }"#,
    )
    .unwrap();

    assert_ir_matches!(
        ir,
        quote! {
            ...
            Namespace {
                name: "test_namespace_bindings" ...
                id: ItemId(...) ...
                canonical_namespace_id: ItemId(...) ...
            }
            ...
        }
    );

    let namespaces = ir.namespaces().collect_vec();
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0].id, namespaces[0].canonical_namespace_id);
    // TODO(rosica): We actually need to have 2 namespaces here, but
    // we currently only generate IR for the canonical decls.
    // Enable the commented out assertion once we generate IR for all
    // namespace segments.
    // assert_eq!(namespaces[0].canonical_namespace_id,
    // namespaces[1].canonical_namespace_id);
}
