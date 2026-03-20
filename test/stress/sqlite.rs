use bic::*;

/// Build a BindingPackage that mirrors the SQLite3 public surface.
/// SQLite has a famously large single-file API (~200 functions, many typedefs).
pub fn sqlite3_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    // --- opaque handle types ---
    for name in ["sqlite3", "sqlite3_stmt", "sqlite3_blob", "sqlite3_backup",
                  "sqlite3_mutex", "sqlite3_vfs", "sqlite3_file",
                  "sqlite3_io_methods", "sqlite3_context", "sqlite3_value"] {
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some(name.into()),
            fields: None, // opaque
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- typedefs ---
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "sqlite3_int64".into(),
        target: BindingType::LongLong,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "sqlite3_uint64".into(),
        target: BindingType::ULongLong,
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "sqlite3_callback".into(),
        target: BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Int),
            parameters: vec![
                BindingType::ptr(BindingType::Void),
                BindingType::Int,
                BindingType::ptr(BindingType::ptr(BindingType::Char)),
                BindingType::ptr(BindingType::ptr(BindingType::Char)),
            ],
            variadic: false,
        },
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "sqlite3_destructor_type".into(),
        target: BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Void),
            parameters: vec![BindingType::ptr(BindingType::Void)],
            variadic: false,
        },
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- functions: lifecycle ---
    let db_ptr = BindingType::ptr(BindingType::RecordRef("sqlite3".into()));
    let db_ptr_ptr = BindingType::ptr(BindingType::ptr(BindingType::RecordRef("sqlite3".into())));
    let stmt_ptr = BindingType::ptr(BindingType::RecordRef("sqlite3_stmt".into()));
    let stmt_ptr_ptr = BindingType::ptr(BindingType::ptr(BindingType::RecordRef("sqlite3_stmt".into())));
    let const_char_ptr = BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() };
    let char_ptr = BindingType::ptr(BindingType::Char);
    let void_ptr = BindingType::ptr(BindingType::Void);

    let functions: Vec<(&str, Vec<(&str, BindingType)>, BindingType, bool)> = vec![
        ("sqlite3_open", vec![("filename", const_char_ptr.clone()), ("ppDb", db_ptr_ptr.clone())], BindingType::Int, false),
        ("sqlite3_open_v2", vec![("filename", const_char_ptr.clone()), ("ppDb", db_ptr_ptr.clone()), ("flags", BindingType::Int), ("zVfs", const_char_ptr.clone())], BindingType::Int, false),
        ("sqlite3_close", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_close_v2", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_exec", vec![("db", db_ptr.clone()), ("sql", const_char_ptr.clone()), ("callback", BindingType::FunctionPointer { return_type: Box::new(BindingType::Int), parameters: vec![void_ptr.clone(), BindingType::Int, BindingType::ptr(BindingType::ptr(BindingType::Char)), BindingType::ptr(BindingType::ptr(BindingType::Char))], variadic: false }), ("arg", void_ptr.clone()), ("errmsg", BindingType::ptr(char_ptr.clone()))], BindingType::Int, false),
        ("sqlite3_errmsg", vec![("db", db_ptr.clone())], const_char_ptr.clone(), false),
        ("sqlite3_errcode", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_extended_errcode", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_errstr", vec![("errcode", BindingType::Int)], const_char_ptr.clone(), false),
        ("sqlite3_changes", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_total_changes", vec![("db", db_ptr.clone())], BindingType::Int, false),
        ("sqlite3_last_insert_rowid", vec![("db", db_ptr.clone())], BindingType::LongLong, false),
        ("sqlite3_busy_timeout", vec![("db", db_ptr.clone()), ("ms", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_busy_handler", vec![("db", db_ptr.clone()), ("handler", BindingType::FunctionPointer { return_type: Box::new(BindingType::Int), parameters: vec![void_ptr.clone(), BindingType::Int], variadic: false }), ("arg", void_ptr.clone())], BindingType::Int, false),
        // prepare / step / finalize
        ("sqlite3_prepare_v2", vec![("db", db_ptr.clone()), ("sql", const_char_ptr.clone()), ("nByte", BindingType::Int), ("ppStmt", stmt_ptr_ptr.clone()), ("pzTail", BindingType::ptr(const_char_ptr.clone()))], BindingType::Int, false),
        ("sqlite3_step", vec![("stmt", stmt_ptr.clone())], BindingType::Int, false),
        ("sqlite3_finalize", vec![("stmt", stmt_ptr.clone())], BindingType::Int, false),
        ("sqlite3_reset", vec![("stmt", stmt_ptr.clone())], BindingType::Int, false),
        // bind
        ("sqlite3_bind_int", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int), ("val", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_bind_int64", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int), ("val", BindingType::LongLong)], BindingType::Int, false),
        ("sqlite3_bind_double", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int), ("val", BindingType::Double)], BindingType::Int, false),
        ("sqlite3_bind_text", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int), ("val", const_char_ptr.clone()), ("n", BindingType::Int), ("destructor", BindingType::FunctionPointer { return_type: Box::new(BindingType::Void), parameters: vec![void_ptr.clone()], variadic: false })], BindingType::Int, false),
        ("sqlite3_bind_blob", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int), ("val", BindingType::Pointer { pointee: Box::new(BindingType::Void), const_pointee: true, qualifiers: TypeQualifiers::default() }), ("n", BindingType::Int), ("destructor", BindingType::FunctionPointer { return_type: Box::new(BindingType::Void), parameters: vec![void_ptr.clone()], variadic: false })], BindingType::Int, false),
        ("sqlite3_bind_null", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_bind_parameter_count", vec![("stmt", stmt_ptr.clone())], BindingType::Int, false),
        ("sqlite3_bind_parameter_index", vec![("stmt", stmt_ptr.clone()), ("name", const_char_ptr.clone())], BindingType::Int, false),
        // column
        ("sqlite3_column_count", vec![("stmt", stmt_ptr.clone())], BindingType::Int, false),
        ("sqlite3_column_type", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_column_int", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_column_int64", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::LongLong, false),
        ("sqlite3_column_double", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Double, false),
        ("sqlite3_column_text", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() }, false),
        ("sqlite3_column_blob", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Pointer { pointee: Box::new(BindingType::Void), const_pointee: true, qualifiers: TypeQualifiers::default() }, false),
        ("sqlite3_column_bytes", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], BindingType::Int, false),
        ("sqlite3_column_name", vec![("stmt", stmt_ptr.clone()), ("idx", BindingType::Int)], const_char_ptr.clone(), false),
        // memory
        ("sqlite3_malloc", vec![("n", BindingType::Int)], void_ptr.clone(), false),
        ("sqlite3_malloc64", vec![("n", BindingType::ULongLong)], void_ptr.clone(), false),
        ("sqlite3_realloc", vec![("ptr", void_ptr.clone()), ("n", BindingType::Int)], void_ptr.clone(), false),
        ("sqlite3_free", vec![("ptr", void_ptr.clone())], BindingType::Void, false),
        // utility
        ("sqlite3_libversion", vec![], const_char_ptr.clone(), false),
        ("sqlite3_libversion_number", vec![], BindingType::Int, false),
        ("sqlite3_threadsafe", vec![], BindingType::Int, false),
        ("sqlite3_sleep", vec![("ms", BindingType::Int)], BindingType::Int, false),
        // printf
        ("sqlite3_mprintf", vec![("fmt", const_char_ptr.clone())], char_ptr.clone(), true),
        ("sqlite3_snprintf", vec![("n", BindingType::Int), ("buf", char_ptr.clone()), ("fmt", const_char_ptr.clone())], char_ptr.clone(), true),
    ];

    for (name, params, ret, variadic) in functions {
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: name.into(),
            calling_convention: CallingConvention::C,
            parameters: params.into_iter().map(|(n, t)| ParameterBinding { name: Some(n.into()), ty: t }).collect(),
            return_type: ret,
            variadic,
            source_offset: None,
        }));
    }

    // --- enums (result codes as an enum) ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("sqlite3_column_type_id".into()),
        variants: vec![
            EnumVariant { name: "SQLITE_INTEGER".into(), value: Some(1) },
            EnumVariant { name: "SQLITE_FLOAT".into(), value: Some(2) },
            EnumVariant { name: "SQLITE_BLOB".into(), value: Some(4) },
            EnumVariant { name: "SQLITE_NULL".into(), value: Some(5) },
            EnumVariant { name: "SQLITE_TEXT".into(), value: Some(3) },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- macros ---
    for (name, val) in [
        ("SQLITE_OK", 0i128), ("SQLITE_ERROR", 1), ("SQLITE_INTERNAL", 2),
        ("SQLITE_PERM", 3), ("SQLITE_ABORT", 4), ("SQLITE_BUSY", 5),
        ("SQLITE_LOCKED", 6), ("SQLITE_NOMEM", 7), ("SQLITE_READONLY", 8),
        ("SQLITE_INTERRUPT", 9), ("SQLITE_IOERR", 10), ("SQLITE_CORRUPT", 11),
        ("SQLITE_NOTFOUND", 12), ("SQLITE_FULL", 13), ("SQLITE_CANTOPEN", 14),
        ("SQLITE_PROTOCOL", 15), ("SQLITE_EMPTY", 16), ("SQLITE_SCHEMA", 17),
        ("SQLITE_TOOBIG", 18), ("SQLITE_CONSTRAINT", 19), ("SQLITE_MISMATCH", 20),
        ("SQLITE_MISUSE", 21), ("SQLITE_NOLFS", 22), ("SQLITE_AUTH", 23),
        ("SQLITE_FORMAT", 24), ("SQLITE_RANGE", 25), ("SQLITE_NOTADB", 26),
        ("SQLITE_ROW", 100), ("SQLITE_DONE", 101),
        ("SQLITE_OPEN_READONLY", 1), ("SQLITE_OPEN_READWRITE", 2),
        ("SQLITE_OPEN_CREATE", 4), ("SQLITE_OPEN_MEMORY", 128),
        ("SQLITE_OPEN_NOMUTEX", 32768), ("SQLITE_OPEN_FULLMUTEX", 65536),
        ("SQLITE_VERSION_NUMBER", 3046000),
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

    pkg.link.libraries.push(LinkLibrary {
        name: "sqlite3".into(),
        kind: LinkLibraryKind::Default,
        source: LinkRequirementSource::Declared,
    });

    pkg
}
