use bic::*;

/// Build a BindingPackage that mirrors the FreeType2 public surface.
/// FreeType is notable for: heavy typedef layering, fixed-point arithmetic
/// types, deeply nested struct hierarchies, and a large stable API.
pub fn freetype_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    let void_ptr = BindingType::ptr(BindingType::Void);
    let const_char_ptr = BindingType::Pointer { pointee: Box::new(BindingType::Char), const_pointee: true, qualifiers: TypeQualifiers::default() };
    let const_uchar_ptr = BindingType::Pointer { pointee: Box::new(BindingType::UChar), const_pointee: true, qualifiers: TypeQualifiers::default() };

    // --- typedefs (FreeType's naming convention) ---
    for (name, target) in [
        ("FT_Error", BindingType::Int),
        ("FT_Int", BindingType::Int),
        ("FT_UInt", BindingType::UInt),
        ("FT_Long", BindingType::Long),
        ("FT_ULong", BindingType::ULong),
        ("FT_Short", BindingType::Short),
        ("FT_UShort", BindingType::UShort),
        ("FT_Byte", BindingType::UChar),
        ("FT_Bool", BindingType::UChar),
        ("FT_Char", BindingType::Char),
        ("FT_String", BindingType::Char),
        ("FT_Fixed", BindingType::Long),    // 16.16 fixed-point
        ("FT_F26Dot6", BindingType::Long),  // 26.6 fixed-point
        ("FT_Pos", BindingType::Long),
        ("FT_Int32", BindingType::Int),
        ("FT_UInt32", BindingType::UInt),
    ] {
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: name.into(),
            target,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- opaque handles ---
    for name in ["FT_LibraryRec", "FT_FaceRec", "FT_SizeRec", "FT_GlyphSlotRec",
                  "FT_CharMapRec", "FT_DriverRec", "FT_MemoryRec", "FT_StreamRec",
                  "FT_SubGlyphRec", "FT_ModuleRec"] {
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some(name.into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // handle typedefs
    for (name, target) in [
        ("FT_Library", "FT_LibraryRec"),
        ("FT_Face", "FT_FaceRec"),
        ("FT_Size", "FT_SizeRec"),
        ("FT_GlyphSlot", "FT_GlyphSlotRec"),
        ("FT_CharMap", "FT_CharMapRec"),
    ] {
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: name.into(),
            target: BindingType::ptr(BindingType::RecordRef(target.into())),
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- by-value structs ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FT_Vector".into()),
        fields: Some(vec![
            FieldBinding { name: Some("x".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("y".into()), ty: BindingType::Long, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FT_BBox".into()),
        fields: Some(vec![
            FieldBinding { name: Some("xMin".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("yMin".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("xMax".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("yMax".into()), ty: BindingType::Long, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FT_Matrix".into()),
        fields: Some(vec![
            FieldBinding { name: Some("xx".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("xy".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("yx".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("yy".into()), ty: BindingType::Long, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FT_Bitmap".into()),
        fields: Some(vec![
            FieldBinding { name: Some("rows".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("width".into()), ty: BindingType::UInt, bit_width: None, layout: None },
            FieldBinding { name: Some("pitch".into()), ty: BindingType::Int, bit_width: None, layout: None },
            FieldBinding { name: Some("buffer".into()), ty: BindingType::ptr(BindingType::UChar), bit_width: None, layout: None },
            FieldBinding { name: Some("num_grays".into()), ty: BindingType::UShort, bit_width: None, layout: None },
            FieldBinding { name: Some("pixel_mode".into()), ty: BindingType::UChar, bit_width: None, layout: None },
            FieldBinding { name: Some("palette_mode".into()), ty: BindingType::UChar, bit_width: None, layout: None },
            FieldBinding { name: Some("palette".into()), ty: void_ptr.clone(), bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FT_Glyph_Metrics".into()),
        fields: Some(vec![
            FieldBinding { name: Some("width".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("height".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("horiBearingX".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("horiBearingY".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("horiAdvance".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("vertBearingX".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("vertBearingY".into()), ty: BindingType::Long, bit_width: None, layout: None },
            FieldBinding { name: Some("vertAdvance".into()), ty: BindingType::Long, bit_width: None, layout: None },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- enums ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("FT_Pixel_Mode".into()),
        variants: vec![
            EnumVariant { name: "FT_PIXEL_MODE_NONE".into(), value: Some(0) },
            EnumVariant { name: "FT_PIXEL_MODE_MONO".into(), value: Some(1) },
            EnumVariant { name: "FT_PIXEL_MODE_GRAY".into(), value: Some(2) },
            EnumVariant { name: "FT_PIXEL_MODE_GRAY2".into(), value: Some(3) },
            EnumVariant { name: "FT_PIXEL_MODE_GRAY4".into(), value: Some(4) },
            EnumVariant { name: "FT_PIXEL_MODE_LCD".into(), value: Some(5) },
            EnumVariant { name: "FT_PIXEL_MODE_LCD_V".into(), value: Some(6) },
            EnumVariant { name: "FT_PIXEL_MODE_BGRA".into(), value: Some(7) },
            EnumVariant { name: "FT_PIXEL_MODE_MAX".into(), value: Some(8) },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("FT_Glyph_Format".into()),
        variants: vec![
            EnumVariant { name: "FT_GLYPH_FORMAT_NONE".into(), value: Some(0) },
            EnumVariant { name: "FT_GLYPH_FORMAT_COMPOSITE".into(), value: Some(0x636F6D70) },
            EnumVariant { name: "FT_GLYPH_FORMAT_BITMAP".into(), value: Some(0x62697473) },
            EnumVariant { name: "FT_GLYPH_FORMAT_OUTLINE".into(), value: Some(0x6F75746C) },
            EnumVariant { name: "FT_GLYPH_FORMAT_PLOTTER".into(), value: Some(0x706C6F74) },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- functions ---
    let lib_ptr = BindingType::ptr(BindingType::RecordRef("FT_LibraryRec".into()));
    let lib_ptr_ptr = BindingType::ptr(lib_ptr.clone());
    let face_ptr = BindingType::ptr(BindingType::RecordRef("FT_FaceRec".into()));
    let face_ptr_ptr = BindingType::ptr(face_ptr.clone());

    let functions: Vec<(&str, Vec<(&str, BindingType)>, BindingType, bool)> = vec![
        ("FT_Init_FreeType", vec![("alibrary", lib_ptr_ptr.clone())], BindingType::Int, false),
        ("FT_Done_FreeType", vec![("library", lib_ptr.clone())], BindingType::Int, false),
        ("FT_New_Face", vec![("library", lib_ptr.clone()), ("filepathname", const_char_ptr.clone()), ("face_index", BindingType::Long), ("aface", face_ptr_ptr.clone())], BindingType::Int, false),
        ("FT_New_Memory_Face", vec![("library", lib_ptr.clone()), ("file_base", const_uchar_ptr.clone()), ("file_size", BindingType::Long), ("face_index", BindingType::Long), ("aface", face_ptr_ptr.clone())], BindingType::Int, false),
        ("FT_Done_Face", vec![("face", face_ptr.clone())], BindingType::Int, false),
        ("FT_Set_Pixel_Sizes", vec![("face", face_ptr.clone()), ("pixel_width", BindingType::UInt), ("pixel_height", BindingType::UInt)], BindingType::Int, false),
        ("FT_Set_Char_Size", vec![("face", face_ptr.clone()), ("char_width", BindingType::Long), ("char_height", BindingType::Long), ("horz_resolution", BindingType::UInt), ("vert_resolution", BindingType::UInt)], BindingType::Int, false),
        ("FT_Load_Glyph", vec![("face", face_ptr.clone()), ("glyph_index", BindingType::UInt), ("load_flags", BindingType::Int)], BindingType::Int, false),
        ("FT_Load_Char", vec![("face", face_ptr.clone()), ("char_code", BindingType::ULong), ("load_flags", BindingType::Int)], BindingType::Int, false),
        ("FT_Render_Glyph", vec![("slot", BindingType::ptr(BindingType::RecordRef("FT_GlyphSlotRec".into()))), ("render_mode", BindingType::Int)], BindingType::Int, false),
        ("FT_Get_Char_Index", vec![("face", face_ptr.clone()), ("charcode", BindingType::ULong)], BindingType::UInt, false),
        ("FT_Get_Kerning", vec![("face", face_ptr.clone()), ("left_glyph", BindingType::UInt), ("right_glyph", BindingType::UInt), ("kern_mode", BindingType::UInt), ("akerning", BindingType::ptr(BindingType::RecordRef("FT_Vector".into())))], BindingType::Int, false),
        ("FT_Select_Charmap", vec![("face", face_ptr.clone()), ("encoding", BindingType::Int)], BindingType::Int, false),
        ("FT_Library_Version", vec![("library", lib_ptr.clone()), ("amajor", BindingType::ptr(BindingType::Int)), ("aminor", BindingType::ptr(BindingType::Int)), ("apatch", BindingType::ptr(BindingType::Int))], BindingType::Void, false),
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

    // macros
    for (name, val) in [
        ("FT_LOAD_DEFAULT", 0i128), ("FT_LOAD_NO_SCALE", 1),
        ("FT_LOAD_NO_HINTING", 2), ("FT_LOAD_RENDER", 4),
        ("FT_LOAD_NO_BITMAP", 8), ("FT_LOAD_FORCE_AUTOHINT", 32),
        ("FT_FACE_FLAG_SCALABLE", 1), ("FT_FACE_FLAG_FIXED_SIZES", 2),
        ("FT_FACE_FLAG_FIXED_WIDTH", 4), ("FT_FACE_FLAG_HORIZONTAL", 16),
        ("FT_FACE_FLAG_VERTICAL", 32), ("FT_FACE_FLAG_KERNING", 64),
        ("FREETYPE_MAJOR", 2), ("FREETYPE_MINOR", 13), ("FREETYPE_PATCH", 2),
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
        name: "freetype".into(),
        kind: LinkLibraryKind::Default,
        source: LinkRequirementSource::Declared,
    });

    pkg
}
