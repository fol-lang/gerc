//! Regression corpus — synthetic and real-world binding fixtures.
//!
//! These tests exercise the full `gec` pipeline: intake → lower → gate → emit.

#[cfg(test)]
mod tests {
    use crate::emit::emit_source;
    use crate::gate::{gate_package, GateDecision};
    use crate::linkgen::lower_link_surface;
    use crate::lower::lower_package;
    use linc::*;

    /// Helper: run the full pipeline on a BindingPackage.
    fn full_pipeline(pkg: &BindingPackage) -> (String, usize, usize) {
        let (decisions, _) = gate_package(pkg, None);
        let accepted = decisions
            .iter()
            .filter(|d| **d == GateDecision::Accept)
            .count();
        let rejected = decisions
            .iter()
            .filter(|d| matches!(d, GateDecision::Reject(_)))
            .count();

        let (mut proj, _diags) = lower_package(pkg);
        proj.link_requirements = lower_link_surface(pkg);
        let source = emit_source(&proj);
        (source, accepted, rejected)
    }

    // 10.1: synthetic torture binding fixture
    #[test]
    fn torture_fixture() {
        let mut pkg = BindingPackage::new();
        // Deep pointer chain
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "deep_ptr".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("p".into()),
                ty: BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::const_ptr(
                    BindingType::Void,
                )))),
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        // Variadic function
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "variadic_fn".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("fmt".into()),
                ty: BindingType::const_ptr(BindingType::Char),
            }],
            return_type: BindingType::Int,
            variadic: true,
            source_offset: None,
        }));
        // Function pointer parameter
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "callback_fn".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("cb".into()),
                ty: BindingType::FunctionPointer {
                    return_type: Box::new(BindingType::Void),
                    parameters: vec![BindingType::Int, BindingType::ptr(BindingType::Void)],
                    variadic: false,
                },
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        // Opaque struct
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("opaque_handle".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Bitfield struct (should be gated)
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("bitfield_struct".into()),
            fields: Some(vec![FieldBinding {
                name: Some("bits".into()),
                ty: BindingType::UInt,
                bit_width: Some(3),
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Anonymous enum (should be gated)
        pkg.items.push(BindingItem::Enum(EnumBinding {
            name: None,
            variants: vec![EnumVariant {
                name: "X".into(),
                value: Some(0),
            }],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Zero-length array
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("flexible_buf".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("len".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("data".into()),
                    ty: BindingType::Array(Box::new(BindingType::UChar), Some(0)),
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Macro constant
        pkg.macros.push(MacroBinding {
            name: "TORTURE_CONST".into(),
            body: "42".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Integer,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::Integer(42)),
        });

        let (source, accepted, rejected) = full_pipeline(&pkg);
        assert!(
            accepted >= 5,
            "expected at least 5 accepted, got {}",
            accepted
        );
        assert!(
            rejected >= 2,
            "expected at least 2 rejected, got {}",
            rejected
        );
        assert!(source.contains("pub fn deep_ptr"));
        assert!(source.contains("pub fn variadic_fn"));
        assert!(source.contains("..."));
        assert!(source.contains("pub fn callback_fn"));
        assert!(source.contains("pub struct opaque_handle"));
        assert!(source.contains("pub struct flexible_buf"));
        assert!(source.contains("pub const TORTURE_CONST"));
    }

    // 10.2: zlib baseline fixture
    #[test]
    fn zlib_fixture() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: "Bytef".into(),
            target: BindingType::UChar,
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
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("z_stream".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("next_in".into()),
                    ty: BindingType::ptr(BindingType::TypedefRef("Bytef".into())),
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("avail_in".into()),
                    ty: BindingType::UInt,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("total_in".into()),
                    ty: BindingType::TypedefRef("uLong".into()),
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "deflateInit".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![
                ParameterBinding {
                    name: Some("strm".into()),
                    ty: BindingType::ptr(BindingType::RecordRef("z_stream".into())),
                },
                ParameterBinding {
                    name: Some("level".into()),
                    ty: BindingType::Int,
                },
            ],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "inflate".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![
                ParameterBinding {
                    name: Some("strm".into()),
                    ty: BindingType::ptr(BindingType::RecordRef("z_stream".into())),
                },
                ParameterBinding {
                    name: Some("flush".into()),
                    ty: BindingType::Int,
                },
            ],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        pkg.macros.push(MacroBinding {
            name: "ZLIB_VERSION".into(),
            body: "\"1.2.13\"".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::String,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::String("1.2.13".into())),
        });

        let (source, accepted, _rejected) = full_pipeline(&pkg);
        assert_eq!(accepted, 5);
        assert!(source.contains("pub type Bytef"));
        assert!(source.contains("pub type uLong"));
        assert!(source.contains("pub struct z_stream"));
        assert!(source.contains("pub fn deflateInit"));
        assert!(source.contains("pub fn inflate"));
        assert!(source.contains("pub const ZLIB_VERSION"));
    }

    // 10.3: SocketCAN/Linux-system fixture
    #[test]
    fn socketcan_fixture() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("can_frame".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("can_id".into()),
                    ty: BindingType::UInt,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("can_dlc".into()),
                    ty: BindingType::UChar,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("data".into()),
                    ty: BindingType::Array(Box::new(BindingType::UChar), Some(8)),
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("sockaddr_can".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("can_family".into()),
                    ty: BindingType::UShort,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("can_ifindex".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.macros.push(MacroBinding {
            name: "CAN_MTU".into(),
            body: "16".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Integer,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::Integer(16)),
        });

        let (source, accepted, _) = full_pipeline(&pkg);
        assert_eq!(accepted, 2);
        assert!(source.contains("pub struct can_frame"));
        assert!(source.contains("[core::ffi::c_uchar; 8]"));
        assert!(source.contains("pub struct sockaddr_can"));
        assert!(source.contains("pub const CAN_MTU"));
    }

    // 10.4: libpcap fixture
    #[test]
    fn libpcap_fixture() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("pcap".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: "pcap_t".into(),
            target: BindingType::RecordRef("pcap".into()),
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "pcap_open_live".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![
                ParameterBinding {
                    name: Some("device".into()),
                    ty: BindingType::const_ptr(BindingType::Char),
                },
                ParameterBinding {
                    name: Some("snaplen".into()),
                    ty: BindingType::Int,
                },
                ParameterBinding {
                    name: Some("promisc".into()),
                    ty: BindingType::Int,
                },
                ParameterBinding {
                    name: Some("to_ms".into()),
                    ty: BindingType::Int,
                },
                ParameterBinding {
                    name: Some("errbuf".into()),
                    ty: BindingType::ptr(BindingType::Char),
                },
            ],
            return_type: BindingType::ptr(BindingType::TypedefRef("pcap_t".into())),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "pcap_close".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("p".into()),
                ty: BindingType::ptr(BindingType::TypedefRef("pcap_t".into())),
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "pcap".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });

        let (source, accepted, _) = full_pipeline(&pkg);
        assert_eq!(accepted, 4);
        assert!(source.contains("pub struct pcap { _opaque:"));
        assert!(source.contains("pub type pcap_t"));
        assert!(source.contains("pub fn pcap_open_live"));
        assert!(source.contains("pub fn pcap_close"));
    }

    // 10.5: libcurl fixture
    #[test]
    fn libcurl_fixture() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("CURL".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Enum(EnumBinding {
            name: Some("CURLcode".into()),
            variants: vec![
                EnumVariant {
                    name: "CURLE_OK".into(),
                    value: Some(0),
                },
                EnumVariant {
                    name: "CURLE_UNSUPPORTED_PROTOCOL".into(),
                    value: Some(1),
                },
                EnumVariant {
                    name: "CURLE_FAILED_INIT".into(),
                    value: Some(2),
                },
            ],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "curl_easy_init".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::ptr(BindingType::RecordRef("CURL".into())),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "curl_easy_cleanup".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("curl".into()),
                ty: BindingType::ptr(BindingType::RecordRef("CURL".into())),
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "curl".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });

        let (source, accepted, _) = full_pipeline(&pkg);
        assert_eq!(accepted, 4);
        assert!(source.contains("pub struct CURL { _opaque:"));
        assert!(source.contains("pub enum CURLcode"));
        assert!(source.contains("CURLE_OK = 0"));
        assert!(source.contains("pub fn curl_easy_init"));
        assert!(source.contains("pub fn curl_easy_cleanup"));
    }

    // 10.6: OpenSSL opaque-surface fixture
    #[test]
    fn openssl_fixture() {
        let mut pkg = BindingPackage::new();
        // OpenSSL is mostly opaque handles
        for name in &["SSL_CTX", "SSL", "SSL_METHOD", "BIO", "X509"] {
            pkg.items.push(BindingItem::Record(RecordBinding {
                kind: RecordKind::Struct,
                name: Some(name.to_string()),
                fields: None,
                representation: None,
                abi_confidence: None,
                source_offset: None,
            }));
        }
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "SSL_CTX_new".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("method".into()),
                ty: BindingType::const_ptr(BindingType::RecordRef("SSL_METHOD".into())),
            }],
            return_type: BindingType::ptr(BindingType::RecordRef("SSL_CTX".into())),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "SSL_new".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("ctx".into()),
                ty: BindingType::ptr(BindingType::RecordRef("SSL_CTX".into())),
            }],
            return_type: BindingType::ptr(BindingType::RecordRef("SSL".into())),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "SSL_free".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("ssl".into()),
                ty: BindingType::ptr(BindingType::RecordRef("SSL".into())),
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "ssl".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });
        pkg.link.libraries.push(LinkLibrary {
            name: "crypto".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });

        let (source, accepted, _) = full_pipeline(&pkg);
        assert_eq!(accepted, 8);
        for name in &["SSL_CTX", "SSL", "SSL_METHOD", "BIO", "X509"] {
            assert!(source.contains(&format!("pub struct {} {{ _opaque:", name)));
        }
        assert!(source.contains("pub fn SSL_CTX_new"));
        assert!(source.contains("pub fn SSL_new"));
        assert!(source.contains("pub fn SSL_free"));
    }

    // 10.7: plugin/runtime-loader boundary fixture
    #[test]
    fn plugin_loader_fixture() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "dlopen".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![
                ParameterBinding {
                    name: Some("filename".into()),
                    ty: BindingType::const_ptr(BindingType::Char),
                },
                ParameterBinding {
                    name: Some("flags".into()),
                    ty: BindingType::Int,
                },
            ],
            return_type: BindingType::ptr(BindingType::Void),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "dlsym".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![
                ParameterBinding {
                    name: Some("handle".into()),
                    ty: BindingType::ptr(BindingType::Void),
                },
                ParameterBinding {
                    name: Some("symbol".into()),
                    ty: BindingType::const_ptr(BindingType::Char),
                },
            ],
            return_type: BindingType::ptr(BindingType::Void),
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "dlclose".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("handle".into()),
                ty: BindingType::ptr(BindingType::Void),
            }],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "dlerror".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::ptr(BindingType::Char),
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "dl".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });

        let (source, accepted, _) = full_pipeline(&pkg);
        assert_eq!(accepted, 4);
        assert!(source.contains("pub fn dlopen"));
        assert!(source.contains("pub fn dlsym"));
        assert!(source.contains("pub fn dlclose"));
        assert!(source.contains("pub fn dlerror"));
        assert!(source.contains("*mut core::ffi::c_void"));
    }

    // 10.8: combined mixed-surface fixture
    #[test]
    fn mixed_surface_fixture() {
        let mut pkg = BindingPackage::new();
        // typedef
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: "size_t".into(),
            target: BindingType::ULong,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // enum
        pkg.items.push(BindingItem::Enum(EnumBinding {
            name: Some("status".into()),
            variants: vec![
                EnumVariant {
                    name: "OK".into(),
                    value: Some(0),
                },
                EnumVariant {
                    name: "ERR".into(),
                    value: Some(1),
                },
            ],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // struct
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("config".into()),
            fields: Some(vec![FieldBinding {
                name: Some("flags".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // union
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Union,
            name: Some("data".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("i".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("f".into()),
                    ty: BindingType::Float,
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // opaque
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("ctx".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // function
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "init".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("c".into()),
                ty: BindingType::ptr(BindingType::RecordRef("config".into())),
            }],
            return_type: BindingType::TypedefRef("status".into()),
            variadic: false,
            source_offset: None,
        }));
        // variable
        pkg.items.push(BindingItem::Variable(VariableBinding {
            name: "errno".into(),
            ty: BindingType::Int,
            source_offset: None,
        }));
        // unsupported
        pkg.items.push(BindingItem::Unsupported(UnsupportedItem {
            name: Some("__attribute__".into()),
            reason: "compiler extension".into(),
            source_offset: None,
        }));
        // macro
        pkg.macros.push(MacroBinding {
            name: "VERSION".into(),
            body: "1".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Integer,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::Integer(1)),
        });

        let (source, accepted, rejected) = full_pipeline(&pkg);
        assert_eq!(accepted, 7);
        assert_eq!(rejected, 1); // unsupported
        assert!(source.contains("pub type size_t"));
        assert!(source.contains("pub enum status"));
        assert!(source.contains("pub struct config"));
        assert!(source.contains("pub union data"));
        assert!(source.contains("pub struct ctx { _opaque:"));
        assert!(source.contains("pub fn init"));
        assert!(source.contains("pub static mut errno"));
        assert!(source.contains("pub const VERSION"));
    }

    // 10.9: emitted Rust is valid syntax (basic check)
    #[test]
    fn emitted_source_has_balanced_braces() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "test_fn".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("S".into()),
            fields: Some(vec![FieldBinding {
                name: Some("x".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Enum(EnumBinding {
            name: Some("E".into()),
            variants: vec![EnumVariant {
                name: "A".into(),
                value: Some(0),
            }],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));

        let (source, _, _) = full_pipeline(&pkg);
        let opens = source.chars().filter(|c| *c == '{').count();
        let closes = source.chars().filter(|c| *c == '}').count();
        assert_eq!(opens, closes, "unbalanced braces in emitted source");
    }

    // 10.10: findings ledger
    #[test]
    fn generation_limitations_ledger() {
        let mut pkg = BindingPackage::new();
        // Bitfield: rejected
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("bf".into()),
            fields: Some(vec![FieldBinding {
                name: Some("x".into()),
                ty: BindingType::UInt,
                bit_width: Some(4),
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Anonymous record: rejected
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: None,
            fields: Some(vec![]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Anonymous enum: rejected
        pkg.items.push(BindingItem::Enum(EnumBinding {
            name: None,
            variants: vec![],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        // Unsupported: rejected
        pkg.items.push(BindingItem::Unsupported(UnsupportedItem {
            name: Some("attr".into()),
            reason: "compiler extension".into(),
            source_offset: None,
        }));
        // long double: mapped to unknown
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "ld_fn".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("x".into()),
                ty: BindingType::LongDouble,
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));

        let (decisions, _) = gate_package(&pkg, None);
        let rejected: Vec<_> = decisions
            .iter()
            .enumerate()
            .filter_map(|(i, d)| match d {
                GateDecision::Reject(r) => Some((i, r.clone())),
                GateDecision::Accept => None,
            })
            .collect();

        // At least 3 rejections: bitfield, anonymous record, anonymous enum, unsupported
        assert!(
            rejected.len() >= 3,
            "expected >=3 rejections, got {}",
            rejected.len()
        );
    }
}
