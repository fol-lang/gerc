use bic::*;

/// Build a BindingPackage that mirrors the zlib public surface.
pub fn zlib_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    // --- core types ---
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "Bytef".into(),
        target: BindingType::UChar,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "uInt".into(),
        target: BindingType::UInt,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "uLong".into(),
        target: BindingType::ULong,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "uLongf".into(),
        target: BindingType::ULong,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "voidpf".into(),
        target: BindingType::ptr(BindingType::Void),
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // z_stream
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("z_stream".into()),
        fields: Some(vec![
            FieldBinding { name: Some("next_in".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("avail_in".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("total_in".into()), ty: BindingType::ULong, bit_width: None, layout: None },
            FieldBinding { name: Some("next_out".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("avail_out".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("total_out".into()), ty: BindingType::ULong, bit_width: None, layout: None },
            FieldBinding { name: Some("msg".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("state".into()), ty: BindingType::ptr(BindingType::Void), bit_width: None, layout: None },
            FieldBinding { name: Some("zalloc".into()), ty: BindingType::FunctionPointer { return_type: Box::new(BindingType::ptr(BindingType::Void)), parameters: vec![BindingType::ptr(BindingType::Void), BindingType::UInt, BindingType::UInt], variadic: false }, bit_width: None, layout: None },
            FieldBinding { name: Some("zfree".into()), ty: BindingType::FunctionPointer { return_type: Box::new(BindingType::Void), parameters: vec![BindingType::ptr(BindingType::Void), BindingType::ptr(BindingType::Void)], variadic: false }, bit_width: None, layout: None },
            FieldBinding { name: Some("opaque".into()), ty: BindingType::ptr(BindingType::Void), bit_width: None, layout: None },
            FieldBinding { name: Some("data_type".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("adler".into()), ty: BindingType::ULong, bit_width: None, layout: None },
            FieldBinding { name: Some("reserved".into()), ty: BindingType::ULong, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // gz_header
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("gz_header".into()),
        fields: Some(vec![
            FieldBinding { name: Some("text".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("time".into()), ty: BindingType::ULong, bit_width: None, layout: None },
            FieldBinding { name: Some("xflags".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("os".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("extra".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("extra_len".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("extra_max".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("name".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("name_max".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("comment".into()), ty: BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }, bit_width: None, layout: None },
            FieldBinding { name: Some("comm_max".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("hcrc".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("done".into()), ty: BindingType::Int, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- functions: compression ---
    for (name, params, ret, variadic) in [
        ("deflateInit_", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into()))), ("level", BindingType::Int), ("version", BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("stream_size", BindingType::Int)], BindingType::Int, false),
        ("deflate", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into()))), ("flush", BindingType::Int)], BindingType::Int, false),
        ("deflateEnd", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into())))], BindingType::Int, false),
        ("inflateInit_", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into()))), ("version", BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("stream_size", BindingType::Int)], BindingType::Int, false),
        ("inflate", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into()))), ("flush", BindingType::Int)], BindingType::Int, false),
        ("inflateEnd", vec![("strm", BindingType::ptr(BindingType::RecordRef("z_stream".into())))], BindingType::Int, false),
        ("compress", vec![("dest", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }), ("destLen", BindingType::ptr(BindingType::ULong)), ("source", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("sourceLen", BindingType::ULong)], BindingType::Int, false),
        ("compress2", vec![("dest", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }), ("destLen", BindingType::ptr(BindingType::ULong)), ("source", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("sourceLen", BindingType::ULong), ("level", BindingType::Int)], BindingType::Int, false),
        ("compressBound", vec![("sourceLen", BindingType::ULong)], BindingType::ULong, false),
        ("uncompress", vec![("dest", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: false, qualifiers: TypeQualifiers::default() }), ("destLen", BindingType::ptr(BindingType::ULong)), ("source", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("sourceLen", BindingType::ULong)], BindingType::Int, false),
        ("adler32", vec![("adler", BindingType::ULong), ("buf", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("len", BindingType::UInt)], BindingType::ULong, false),
        ("crc32", vec![("crc", BindingType::ULong), ("buf", BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("len", BindingType::UInt)], BindingType::ULong, false),
        ("zlibVersion", vec![], BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() }, false),
        ("zlibCompileFlags", vec![], BindingType::ULong, false),
    ] {
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: name.into(),
            calling_convention: CallingConvention::C,
            parameters: params.into_iter().map(|(n, t)| ParameterBinding { name: Some(n.into()), ty: t }).collect(),
            return_type: ret,
            variadic,
            source_offset: None,
        }));
    }

    // --- macros ---
    for (name, val) in [
        ("Z_OK", 0i128), ("Z_STREAM_END", 1), ("Z_NEED_DICT", 2),
        ("Z_ERRNO", -1), ("Z_STREAM_ERROR", -2), ("Z_DATA_ERROR", -3),
        ("Z_MEM_ERROR", -4), ("Z_BUF_ERROR", -5), ("Z_VERSION_ERROR", -6),
        ("Z_NO_COMPRESSION", 0), ("Z_BEST_SPEED", 1),
        ("Z_BEST_COMPRESSION", 9), ("Z_DEFAULT_COMPRESSION", -1),
        ("Z_DEFLATED", 8),
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
    // ZLIB_VERSION is a string macro — not bindable
    pkg.macros.push(MacroBinding {
        name: "ZLIB_VERSION".into(),
        body: "\"1.3.1\"".into(),
        function_like: false,
        form: MacroForm::ObjectLike,
        kind: MacroKind::String,
        category: MacroCategory::Unsupported,
        value: None,
    });

    // link surface
    pkg.link.libraries.push(LinkLibrary {
        name: "z".into(),
        kind: LinkLibraryKind::Default,
        source: LinkRequirementSource::Declared,
    });

    pkg
}
