use bic::*;

/// Build a pathological BindingPackage designed to exercise every edge case
/// in the gec pipeline: mixed accept/reject items, deep pointer chains,
/// variadic functions, function pointer callbacks, flexible arrays, opaque
/// types, anonymous (rejected) types, bitfields (rejected), unsupported items,
/// and large enum spaces.
pub fn torture_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    let void_ptr = BindingType::ptr(BindingType::Void);
    let const_char_ptr = BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() };

    // --- deeply nested pointer (5 levels) ---
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "deep_ptr_fn".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![ParameterBinding {
            name: Some("p".into()),
            ty: BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::Int))))),
        }],
        return_type: BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::Void))),
        variadic: false,
        source_offset: None,
    }));

    // --- function that takes a callback that itself takes a callback ---
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "nested_callback".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![ParameterBinding {
            name: Some("cb".into()),
            ty: BindingType::FunctionPointer {
                return_type: Box::new(BindingType::Void),
                parameters: vec![BindingType::FunctionPointer {
                    return_type: Box::new(BindingType::Int),
                    parameters: vec![BindingType::ptr(BindingType::Void), BindingType::Int],
                    variadic: false,
                }],
                variadic: false,
            },
        }],
        return_type: BindingType::Void,
        variadic: false,
        source_offset: None,
    }));

    // --- variadic with many params ---
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "torture_printf".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding { name: Some("fd".into()), ty: BindingType::Int },
            ParameterBinding { name: Some("fmt".into()), ty: const_char_ptr.clone() },
        ],
        return_type: BindingType::Int,
        variadic: true,
        source_offset: None,
    }));

    // --- function with no parameters, returning void ---
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "torture_noop".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Void,
        variadic: false,
        source_offset: None,
    }));

    // --- function with unnamed parameters ---
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "unnamed_params".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding { name: None, ty: BindingType::Int },
            ParameterBinding { name: None, ty: BindingType::ptr(BindingType::Void) },
            ParameterBinding { name: None, ty: BindingType::Double },
        ],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));

    // --- struct with flexible array member ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("flexible_msg".into()),
        fields: Some(vec![
            FieldBinding { name: Some("len".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("data".into()), ty: BindingType::Array(Box::new(BindingType::UChar), None), bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- struct with all primitive types ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("all_primitives".into()),
        fields: Some(vec![
            FieldBinding { name: Some("f_bool".into()), ty: BindingType::Bool, bit_width: None, layout: None },
            FieldBinding { name: Some("f_char".into()), ty: BindingType::Char, bit_width: None, layout: None },
            FieldBinding { name: Some("f_schar".into()), ty: BindingType::SChar, bit_width: None, layout: None },
            FieldBinding { name: Some("f_uchar".into()), ty: BindingType::UChar, bit_width: None, layout: None },
            FieldBinding { name: Some("f_short".into()), ty: BindingType::Short, bit_width: None, layout: None },
            FieldBinding { name: Some("f_ushort".into()), ty: BindingType::UShort, bit_width: None, layout: None },
            FieldBinding { name: Some("f_int".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("f_uint".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("f_long".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("f_ulong".into()), ty: BindingType::ULong, bit_width: None, layout: None },
            FieldBinding { name: Some("f_longlong".into()), ty: BindingType::LongLong, bit_width: None, layout: None },
            FieldBinding { name: Some("f_ulonglong".into()), ty: BindingType::ULongLong, bit_width: None, layout: None },
            FieldBinding { name: Some("f_float".into()), ty: BindingType::Float, bit_width: None, layout: None },
            FieldBinding { name: Some("f_double".into()), ty: BindingType::Double, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- union ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Union,
        name: Some("torture_union".into()),
        fields: Some(vec![
            FieldBinding { name: Some("i".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("f".into()), ty: BindingType::Float, bit_width: None, layout: None },
            FieldBinding { name: Some("p".into()), ty: void_ptr.clone(), bit_width: None, layout: None },
            FieldBinding { name: Some("arr".into()), ty: BindingType::Array(Box::new(BindingType::UChar), Some(16)), bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- opaque struct ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("torture_opaque".into()),
        fields: None,
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- REJECTED: anonymous record ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: None,
        fields: Some(vec![
            FieldBinding { name: Some("x".into()), ty: BindingType::Int, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- REJECTED: bitfield record ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("bitfield_torture".into()),
        fields: Some(vec![
            FieldBinding { name: Some("a".into()), ty: BindingType::UInt, bit_width: Some(3), layout: None },
            FieldBinding { name: Some("b".into()), ty: BindingType::UInt, bit_width: Some(5), layout: None },
            FieldBinding { name: Some("c".into()), ty: BindingType::UInt, bit_width: Some(24), layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- REJECTED: anonymous enum ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: None,
        variants: vec![
            EnumVariant { name: "ANON_A".into(), value: Some(0) },
            EnumVariant { name: "ANON_B".into(), value: Some(1) },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- large enum (50 variants) ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("torture_big_enum".into()),
        variants: (0..50).map(|i| EnumVariant {
            name: format!("TORTURE_VARIANT_{i}"),
            value: Some(i),
        }).collect(),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- enum with negative values ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("torture_signed_enum".into()),
        variants: vec![
            EnumVariant { name: "NEG_THREE".into(), value: Some(-3) },
            EnumVariant { name: "NEG_TWO".into(), value: Some(-2) },
            EnumVariant { name: "NEG_ONE".into(), value: Some(-1) },
            EnumVariant { name: "ZERO".into(), value: Some(0) },
            EnumVariant { name: "POS_ONE".into(), value: Some(1) },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- typedef chain ---
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "torture_handle".into(),
        target: BindingType::ptr(BindingType::RecordRef("torture_opaque".into())),
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "torture_const_handle".into(),
        target: BindingType::Pointer { pointee: Box::new(BindingType::RecordRef("torture_opaque".into())), const_pointee: true, qualifiers: TypeQualifiers::default() },
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "torture_callback_t".into(),
        target: BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Int),
            parameters: vec![void_ptr.clone(), BindingType::ULong, const_char_ptr.clone()],
            variadic: false,
        },
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- variable declarations ---
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "torture_global_state".into(),
        ty: BindingType::Int,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "torture_version_string".into(),
        ty: const_char_ptr.clone(),
        source_offset: None,
    }));

    // --- unsupported item ---
    pkg.items.push(BindingItem::Unsupported(UnsupportedItem {
        name: Some("__torture_internal".into()),
        reason: "compiler-specific builtin, not projectable".into(),
        source_offset: None,
    }));

    // --- macros: mix of bindable and non-bindable ---
    // Integer macros
    for (name, val) in [
        ("TORTURE_VERSION", 42i128), ("TORTURE_MAGIC", 0xDEADBEEFi128),
        ("TORTURE_MAX_SIZE", 65536),
    ] {
        pkg.macros.push(MacroBinding {
            name: name.into(),
            body: val.to_string(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Integer,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::Integer(val)),
        });
    }
    // TORTURE_PI is a float expression — not bindable
    pkg.macros.push(MacroBinding {
        name: "TORTURE_PI".into(),
        body: "3.14159".into(),
        function_like: false,
        form: MacroForm::ObjectLike,
        kind: MacroKind::Expression,
        category: MacroCategory::Unsupported,
        value: None,
    });
    // TORTURE_GREETING is a string — not bindable
    pkg.macros.push(MacroBinding {
        name: "TORTURE_GREETING".into(),
        body: "\"hello\"".into(),
        function_like: false,
        form: MacroForm::ObjectLike,
        kind: MacroKind::String,
        category: MacroCategory::Unsupported,
        value: None,
    });
    // function-like macro (not bindable)
    pkg.macros.push(MacroBinding {
        name: "TORTURE_MAX".into(),
        body: "(a > b ? a : b)".into(),
        function_like: true,
        form: MacroForm::FunctionLike,
        kind: MacroKind::Expression,
        category: MacroCategory::Unsupported,
        value: None,
    });

    // link
    pkg.link.libraries.push(LinkLibrary {
        name: "torture".into(),
        kind: LinkLibraryKind::Default,
        source: LinkRequirementSource::Declared,
    });

    pkg
}
