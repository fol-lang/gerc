use linc::*;

/// Build a BindingPackage that mirrors the OpenSSL public surface.
/// OpenSSL is notable for: almost everything opaque, heavy typedef layering,
/// deprecated callback patterns, and two link libraries (ssl + crypto).
pub fn openssl_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    let void_ptr = BindingType::ptr(BindingType::Void);
    let const_void_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::Void),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let const_char_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::Char),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let const_uchar_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::UChar),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let uchar_ptr = BindingType::ptr(BindingType::UChar);
    let char_ptr = BindingType::ptr(BindingType::Char);

    // --- opaque types (OpenSSL hides almost everything) ---
    for name in [
        "SSL",
        "SSL_CTX",
        "SSL_METHOD",
        "BIO",
        "BIO_METHOD",
        "X509",
        "X509_STORE",
        "X509_STORE_CTX",
        "X509_NAME",
        "EVP_PKEY",
        "EVP_MD",
        "EVP_MD_CTX",
        "EVP_CIPHER",
        "EVP_CIPHER_CTX",
        "ENGINE",
        "RSA",
        "DSA",
        "DH",
        "EC_KEY",
        "BIGNUM",
        "BN_CTX",
        "ASN1_INTEGER",
        "ASN1_TIME",
        "STACK_OF_X509",
        "PKCS12",
        "HMAC_CTX",
        "SSL_SESSION",
        "OSSL_LIB_CTX",
        "OSSL_PROVIDER",
        "EVP_MAC",
        "EVP_MAC_CTX",
        "EVP_KDF",
        "EVP_KDF_CTX",
    ] {
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some(name.into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- typedefs ---
    for (name, target) in [
        ("SSL_CTX", BindingType::RecordRef("SSL_CTX".into())),
        ("SSL", BindingType::RecordRef("SSL".into())),
        (
            "pem_password_cb",
            BindingType::FunctionPointer {
                return_type: Box::new(BindingType::Int),
                parameters: vec![
                    char_ptr.clone(),
                    BindingType::Int,
                    BindingType::Int,
                    void_ptr.clone(),
                ],
                variadic: false,
            },
        ),
        (
            "SSL_verify_cb",
            BindingType::FunctionPointer {
                return_type: Box::new(BindingType::Int),
                parameters: vec![
                    BindingType::Int,
                    BindingType::ptr(BindingType::RecordRef("X509_STORE_CTX".into())),
                ],
                variadic: false,
            },
        ),
    ] {
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: format!("{name}_t"),
            target,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    let ssl_ptr = BindingType::ptr(BindingType::RecordRef("SSL".into()));
    let ssl_ctx_ptr = BindingType::ptr(BindingType::RecordRef("SSL_CTX".into()));
    let ssl_method_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("SSL_METHOD".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let bio_ptr = BindingType::ptr(BindingType::RecordRef("BIO".into()));
    let bio_method_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("BIO_METHOD".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let x509_ptr = BindingType::ptr(BindingType::RecordRef("X509".into()));
    let _evp_pkey_ptr = BindingType::ptr(BindingType::RecordRef("EVP_PKEY".into()));
    let evp_md_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("EVP_MD".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let evp_md_ctx_ptr = BindingType::ptr(BindingType::RecordRef("EVP_MD_CTX".into()));
    let evp_cipher_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("EVP_CIPHER".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let evp_cipher_ctx_ptr = BindingType::ptr(BindingType::RecordRef("EVP_CIPHER_CTX".into()));
    let bn_ptr = BindingType::ptr(BindingType::RecordRef("BIGNUM".into()));

    let functions: Vec<(&str, Vec<(&str, BindingType)>, BindingType, bool)> = vec![
        // SSL lifecycle
        (
            "SSL_CTX_new",
            vec![("method", ssl_method_ptr.clone())],
            ssl_ctx_ptr.clone(),
            false,
        ),
        (
            "SSL_CTX_free",
            vec![("ctx", ssl_ctx_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "SSL_new",
            vec![("ctx", ssl_ctx_ptr.clone())],
            ssl_ptr.clone(),
            false,
        ),
        (
            "SSL_free",
            vec![("ssl", ssl_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "SSL_set_fd",
            vec![("ssl", ssl_ptr.clone()), ("fd", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "SSL_connect",
            vec![("ssl", ssl_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "SSL_accept",
            vec![("ssl", ssl_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "SSL_read",
            vec![
                ("ssl", ssl_ptr.clone()),
                ("buf", void_ptr.clone()),
                ("num", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_write",
            vec![
                ("ssl", ssl_ptr.clone()),
                ("buf", const_void_ptr.clone()),
                ("num", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_shutdown",
            vec![("ssl", ssl_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "SSL_get_error",
            vec![
                (
                    "ssl",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("SSL".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("ret_code", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_get_peer_certificate",
            vec![(
                "ssl",
                BindingType::Pointer {
                    pointee: Box::new(BindingType::RecordRef("SSL".into())),
                    const_pointee: true,
                    qualifiers: TypeQualifiers::default(),
                },
            )],
            x509_ptr.clone(),
            false,
        ),
        // context config
        (
            "SSL_CTX_set_verify",
            vec![
                ("ctx", ssl_ctx_ptr.clone()),
                ("mode", BindingType::Int),
                (
                    "callback",
                    BindingType::FunctionPointer {
                        return_type: Box::new(BindingType::Int),
                        parameters: vec![
                            BindingType::Int,
                            BindingType::ptr(BindingType::RecordRef("X509_STORE_CTX".into())),
                        ],
                        variadic: false,
                    },
                ),
            ],
            BindingType::Void,
            false,
        ),
        (
            "SSL_CTX_load_verify_locations",
            vec![
                ("ctx", ssl_ctx_ptr.clone()),
                ("CAfile", const_char_ptr.clone()),
                ("CApath", const_char_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_CTX_use_certificate_file",
            vec![
                ("ctx", ssl_ctx_ptr.clone()),
                ("file", const_char_ptr.clone()),
                ("type_", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_CTX_use_PrivateKey_file",
            vec![
                ("ctx", ssl_ctx_ptr.clone()),
                ("file", const_char_ptr.clone()),
                ("type_", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "SSL_CTX_check_private_key",
            vec![(
                "ctx",
                BindingType::Pointer {
                    pointee: Box::new(BindingType::RecordRef("SSL_CTX".into())),
                    const_pointee: true,
                    qualifiers: TypeQualifiers::default(),
                },
            )],
            BindingType::Int,
            false,
        ),
        // methods
        ("TLS_method", vec![], ssl_method_ptr.clone(), false),
        ("TLS_client_method", vec![], ssl_method_ptr.clone(), false),
        ("TLS_server_method", vec![], ssl_method_ptr.clone(), false),
        // BIO
        (
            "BIO_new",
            vec![("type_", bio_method_ptr.clone())],
            bio_ptr.clone(),
            false,
        ),
        (
            "BIO_free",
            vec![("a", bio_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "BIO_read",
            vec![
                ("b", bio_ptr.clone()),
                ("data", void_ptr.clone()),
                ("dlen", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "BIO_write",
            vec![
                ("b", bio_ptr.clone()),
                ("data", const_void_ptr.clone()),
                ("dlen", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        ("BIO_s_mem", vec![], bio_method_ptr.clone(), false),
        // EVP digest
        ("EVP_MD_CTX_new", vec![], evp_md_ctx_ptr.clone(), false),
        (
            "EVP_MD_CTX_free",
            vec![("ctx", evp_md_ctx_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "EVP_DigestInit_ex",
            vec![
                ("ctx", evp_md_ctx_ptr.clone()),
                ("type_", evp_md_ptr.clone()),
                (
                    "impl_",
                    BindingType::ptr(BindingType::RecordRef("ENGINE".into())),
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "EVP_DigestUpdate",
            vec![
                ("ctx", evp_md_ctx_ptr.clone()),
                ("d", const_void_ptr.clone()),
                ("cnt", BindingType::ULong),
            ],
            BindingType::Int,
            false,
        ),
        (
            "EVP_DigestFinal_ex",
            vec![
                ("ctx", evp_md_ctx_ptr.clone()),
                ("md", uchar_ptr.clone()),
                ("s", BindingType::ptr(BindingType::UInt)),
            ],
            BindingType::Int,
            false,
        ),
        ("EVP_sha256", vec![], evp_md_ptr.clone(), false),
        ("EVP_sha384", vec![], evp_md_ptr.clone(), false),
        ("EVP_sha512", vec![], evp_md_ptr.clone(), false),
        // EVP cipher
        (
            "EVP_CIPHER_CTX_new",
            vec![],
            evp_cipher_ctx_ptr.clone(),
            false,
        ),
        (
            "EVP_CIPHER_CTX_free",
            vec![("c", evp_cipher_ctx_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "EVP_EncryptInit_ex",
            vec![
                ("ctx", evp_cipher_ctx_ptr.clone()),
                ("cipher", evp_cipher_ptr.clone()),
                (
                    "impl_",
                    BindingType::ptr(BindingType::RecordRef("ENGINE".into())),
                ),
                ("key", const_uchar_ptr.clone()),
                ("iv", const_uchar_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "EVP_EncryptUpdate",
            vec![
                ("ctx", evp_cipher_ctx_ptr.clone()),
                ("out", uchar_ptr.clone()),
                ("outl", BindingType::ptr(BindingType::Int)),
                ("in_", const_uchar_ptr.clone()),
                ("inl", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "EVP_EncryptFinal_ex",
            vec![
                ("ctx", evp_cipher_ctx_ptr.clone()),
                ("out", uchar_ptr.clone()),
                ("outl", BindingType::ptr(BindingType::Int)),
            ],
            BindingType::Int,
            false,
        ),
        ("EVP_aes_256_cbc", vec![], evp_cipher_ptr.clone(), false),
        ("EVP_aes_256_gcm", vec![], evp_cipher_ptr.clone(), false),
        // X509
        (
            "X509_free",
            vec![("a", x509_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "X509_get_subject_name",
            vec![(
                "a",
                BindingType::Pointer {
                    pointee: Box::new(BindingType::RecordRef("X509".into())),
                    const_pointee: true,
                    qualifiers: TypeQualifiers::default(),
                },
            )],
            BindingType::ptr(BindingType::RecordRef("X509_NAME".into())),
            false,
        ),
        (
            "X509_NAME_oneline",
            vec![
                (
                    "a",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("X509_NAME".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("buf", char_ptr.clone()),
                ("size", BindingType::Int),
            ],
            char_ptr.clone(),
            false,
        ),
        // BIGNUM
        ("BN_new", vec![], bn_ptr.clone(), false),
        (
            "BN_free",
            vec![("a", bn_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "BN_num_bits",
            vec![(
                "a",
                BindingType::Pointer {
                    pointee: Box::new(BindingType::RecordRef("BIGNUM".into())),
                    const_pointee: true,
                    qualifiers: TypeQualifiers::default(),
                },
            )],
            BindingType::Int,
            false,
        ),
        // error
        ("ERR_get_error", vec![], BindingType::ULong, false),
        (
            "ERR_error_string_n",
            vec![
                ("e", BindingType::ULong),
                ("buf", char_ptr.clone()),
                ("len", BindingType::ULong),
            ],
            BindingType::Void,
            false,
        ),
        // init
        (
            "OPENSSL_init_ssl",
            vec![
                ("opts", BindingType::ULongLong),
                ("settings", void_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "OPENSSL_init_crypto",
            vec![
                ("opts", BindingType::ULongLong),
                ("settings", void_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "OpenSSL_version",
            vec![("type_", BindingType::Int)],
            const_char_ptr.clone(),
            false,
        ),
        ("OpenSSL_version_num", vec![], BindingType::ULong, false),
    ];

    for (name, params, ret, variadic) in functions {
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: name.into(),
            calling_convention: CallingConvention::C,
            parameters: params
                .into_iter()
                .map(|(n, t)| ParameterBinding {
                    name: Some(n.into()),
                    ty: t,
                })
                .collect(),
            return_type: ret,
            variadic,
            source_offset: None,
        }));
    }

    // --- macros ---
    for (name, val) in [
        ("OPENSSL_VERSION_NUMBER", 0x30200000i128),
        ("SSL_FILETYPE_PEM", 1),
        ("SSL_FILETYPE_ASN1", 2),
        ("SSL_VERIFY_NONE", 0),
        ("SSL_VERIFY_PEER", 1),
        ("SSL_VERIFY_FAIL_IF_NO_PEER_CERT", 2),
        ("SSL_ERROR_NONE", 0),
        ("SSL_ERROR_SSL", 1),
        ("SSL_ERROR_WANT_READ", 2),
        ("SSL_ERROR_WANT_WRITE", 3),
        ("SSL_ERROR_SYSCALL", 5),
        ("SSL_ERROR_ZERO_RETURN", 6),
        ("EVP_MAX_MD_SIZE", 64),
        ("EVP_MAX_KEY_LENGTH", 64),
        ("EVP_MAX_IV_LENGTH", 16),
        ("EVP_MAX_BLOCK_LENGTH", 32),
        ("NID_X9_62_prime256v1", 415),
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

    // link
    for lib in ["ssl", "crypto"] {
        pkg.link.libraries.push(LinkLibrary {
            name: lib.into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
    }

    pkg
}
