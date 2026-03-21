use linc::*;

/// Build a BindingPackage that mirrors a significant slice of FFmpeg's
/// libavcodec + libavformat + libavutil public surface.
///
/// FFmpeg is one of the largest C API surfaces in the wild: ~500+ public
/// functions, deep struct hierarchies, function pointer tables, variadic
/// logging, and complex enum spaces.
pub fn ffmpeg_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();

    let void_ptr = BindingType::ptr(BindingType::Void);
    let const_char_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::Char),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let uint8_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::UChar),
        const_pointee: false,
        qualifiers: TypeQualifiers::default(),
    };
    let const_uint8_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::UChar),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };

    // --- opaque handle types ---
    for name in [
        "AVFormatContext",
        "AVCodecContext",
        "AVCodec",
        "AVStream",
        "AVPacket",
        "AVFrame",
        "AVDictionary",
        "AVIOContext",
        "SwsContext",
        "SwrContext",
        "AVFilterGraph",
        "AVFilterContext",
        "AVFilter",
        "AVBufferRef",
        "AVClass",
        "AVCodecParameters",
        "AVInputFormat",
        "AVOutputFormat",
        "AVCodecHWConfig",
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

    // --- key by-value structs ---
    // AVRational
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("AVRational".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("num".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("den".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // AVDictionaryEntry
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("AVDictionaryEntry".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("key".into()),
                ty: BindingType::ptr(BindingType::Char),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("value".into()),
                ty: BindingType::ptr(BindingType::Char),
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // AVProbeData
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("AVProbeData".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("filename".into()),
                ty: const_char_ptr.clone(),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("buf".into()),
                ty: BindingType::Pointer {
                    pointee: Box::new(BindingType::UChar),
                    const_pointee: false,
                    qualifiers: TypeQualifiers::default(),
                },
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("buf_size".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("mime_type".into()),
                ty: const_char_ptr.clone(),
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- enums ---
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("AVCodecID".into()),
        variants: {
            let mut v = Vec::new();
            for (name, val) in [
                ("AV_CODEC_ID_NONE", 0),
                ("AV_CODEC_ID_MPEG1VIDEO", 1),
                ("AV_CODEC_ID_MPEG2VIDEO", 2),
                ("AV_CODEC_ID_H261", 3),
                ("AV_CODEC_ID_H263", 4),
                ("AV_CODEC_ID_RV10", 5),
                ("AV_CODEC_ID_RV20", 6),
                ("AV_CODEC_ID_MJPEG", 7),
                ("AV_CODEC_ID_MJPEGB", 8),
                ("AV_CODEC_ID_LJPEG", 9),
                ("AV_CODEC_ID_SP5X", 10),
                ("AV_CODEC_ID_JPEGLS", 11),
                ("AV_CODEC_ID_MPEG4", 12),
                ("AV_CODEC_ID_RAWVIDEO", 13),
                ("AV_CODEC_ID_MSMPEG4V1", 14),
                ("AV_CODEC_ID_MSMPEG4V2", 15),
                ("AV_CODEC_ID_MSMPEG4V3", 16),
                ("AV_CODEC_ID_WMV1", 17),
                ("AV_CODEC_ID_WMV2", 18),
                ("AV_CODEC_ID_H263P", 19),
                ("AV_CODEC_ID_H263I", 20),
                ("AV_CODEC_ID_FLV1", 21),
                ("AV_CODEC_ID_SVQ1", 22),
                ("AV_CODEC_ID_SVQ3", 23),
                ("AV_CODEC_ID_DVVIDEO", 24),
                ("AV_CODEC_ID_HUFFYUV", 25),
                ("AV_CODEC_ID_H264", 27),
                ("AV_CODEC_ID_VP3", 35),
                ("AV_CODEC_ID_THEORA", 36),
                ("AV_CODEC_ID_VP5", 60),
                ("AV_CODEC_ID_VP6", 61),
                ("AV_CODEC_ID_VP8", 139),
                ("AV_CODEC_ID_VP9", 167),
                ("AV_CODEC_ID_HEVC", 173),
                ("AV_CODEC_ID_AV1", 226),
            ] {
                v.push(EnumVariant {
                    name: name.into(),
                    value: Some(val),
                });
            }
            v
        },
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("AVPixelFormat".into()),
        variants: {
            let mut v = Vec::new();
            for (name, val) in [
                ("AV_PIX_FMT_NONE", -1),
                ("AV_PIX_FMT_YUV420P", 0),
                ("AV_PIX_FMT_YUYV422", 1),
                ("AV_PIX_FMT_RGB24", 2),
                ("AV_PIX_FMT_BGR24", 3),
                ("AV_PIX_FMT_YUV422P", 4),
                ("AV_PIX_FMT_YUV444P", 5),
                ("AV_PIX_FMT_YUV410P", 6),
                ("AV_PIX_FMT_YUV411P", 7),
                ("AV_PIX_FMT_GRAY8", 8),
                ("AV_PIX_FMT_MONOWHITE", 9),
                ("AV_PIX_FMT_MONOBLACK", 10),
                ("AV_PIX_FMT_PAL8", 11),
                ("AV_PIX_FMT_YUVJ420P", 12),
                ("AV_PIX_FMT_YUVJ422P", 13),
                ("AV_PIX_FMT_YUVJ444P", 14),
                ("AV_PIX_FMT_NV12", 23),
                ("AV_PIX_FMT_NV21", 24),
                ("AV_PIX_FMT_ARGB", 25),
                ("AV_PIX_FMT_RGBA", 26),
                ("AV_PIX_FMT_ABGR", 27),
                ("AV_PIX_FMT_BGRA", 28),
                ("AV_PIX_FMT_RGB48BE", 29),
                ("AV_PIX_FMT_NB", 200),
            ] {
                v.push(EnumVariant {
                    name: name.into(),
                    value: Some(val),
                });
            }
            v
        },
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("AVSampleFormat".into()),
        variants: {
            let mut v = Vec::new();
            for (name, val) in [
                ("AV_SAMPLE_FMT_NONE", -1),
                ("AV_SAMPLE_FMT_U8", 0),
                ("AV_SAMPLE_FMT_S16", 1),
                ("AV_SAMPLE_FMT_S32", 2),
                ("AV_SAMPLE_FMT_FLT", 3),
                ("AV_SAMPLE_FMT_DBL", 4),
                ("AV_SAMPLE_FMT_U8P", 5),
                ("AV_SAMPLE_FMT_S16P", 6),
                ("AV_SAMPLE_FMT_S32P", 7),
                ("AV_SAMPLE_FMT_FLTP", 8),
                ("AV_SAMPLE_FMT_DBLP", 9),
                ("AV_SAMPLE_FMT_S64", 10),
                ("AV_SAMPLE_FMT_S64P", 11),
                ("AV_SAMPLE_FMT_NB", 12),
            ] {
                v.push(EnumVariant {
                    name: name.into(),
                    value: Some(val),
                });
            }
            v
        },
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("AVMediaType".into()),
        variants: vec![
            EnumVariant {
                name: "AVMEDIA_TYPE_UNKNOWN".into(),
                value: Some(-1),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_VIDEO".into(),
                value: Some(0),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_AUDIO".into(),
                value: Some(1),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_DATA".into(),
                value: Some(2),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_SUBTITLE".into(),
                value: Some(3),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_ATTACHMENT".into(),
                value: Some(4),
            },
            EnumVariant {
                name: "AVMEDIA_TYPE_NB".into(),
                value: Some(5),
            },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- typedefs ---
    for (name, target) in [
        ("AVCodecID", BindingType::EnumRef("AVCodecID".into())),
        (
            "AVPixelFormat",
            BindingType::EnumRef("AVPixelFormat".into()),
        ),
    ] {
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: format!("enum_{name}"),
            target,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- functions: libavformat ---
    let fmt_ctx_ptr = BindingType::ptr(BindingType::RecordRef("AVFormatContext".into()));
    let fmt_ctx_ptr_ptr = BindingType::ptr(fmt_ctx_ptr.clone());
    let dict_ptr_ptr = BindingType::ptr(BindingType::ptr(BindingType::RecordRef(
        "AVDictionary".into(),
    )));
    let pkt_ptr = BindingType::ptr(BindingType::RecordRef("AVPacket".into()));
    let frame_ptr = BindingType::ptr(BindingType::RecordRef("AVFrame".into()));
    let codec_ctx_ptr = BindingType::ptr(BindingType::RecordRef("AVCodecContext".into()));
    let codec_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("AVCodec".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };
    let codec_params_ptr = BindingType::Pointer {
        pointee: Box::new(BindingType::RecordRef("AVCodecParameters".into())),
        const_pointee: true,
        qualifiers: TypeQualifiers::default(),
    };

    let functions: Vec<(&str, Vec<(&str, BindingType)>, BindingType, bool)> = vec![
        // avformat
        (
            "avformat_open_input",
            vec![
                ("ps", fmt_ctx_ptr_ptr.clone()),
                ("url", const_char_ptr.clone()),
                (
                    "fmt",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVInputFormat".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("options", dict_ptr_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avformat_close_input",
            vec![("s", fmt_ctx_ptr_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "avformat_find_stream_info",
            vec![
                ("ic", fmt_ctx_ptr.clone()),
                ("options", dict_ptr_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "av_read_frame",
            vec![("s", fmt_ctx_ptr.clone()), ("pkt", pkt_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "av_seek_frame",
            vec![
                ("s", fmt_ctx_ptr.clone()),
                ("stream_index", BindingType::Int),
                ("timestamp", BindingType::LongLong),
                ("flags", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        ("avformat_alloc_context", vec![], fmt_ctx_ptr.clone(), false),
        (
            "avformat_free_context",
            vec![("s", fmt_ctx_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "avformat_alloc_output_context2",
            vec![
                ("ctx", fmt_ctx_ptr_ptr.clone()),
                (
                    "oformat",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVOutputFormat".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("format_name", const_char_ptr.clone()),
                ("filename", const_char_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avformat_write_header",
            vec![
                ("s", fmt_ctx_ptr.clone()),
                ("options", dict_ptr_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "av_write_frame",
            vec![("s", fmt_ctx_ptr.clone()), ("pkt", pkt_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "av_interleaved_write_frame",
            vec![("s", fmt_ctx_ptr.clone()), ("pkt", pkt_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "av_write_trailer",
            vec![("s", fmt_ctx_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "av_dump_format",
            vec![
                ("ic", fmt_ctx_ptr.clone()),
                ("index", BindingType::Int),
                ("url", const_char_ptr.clone()),
                ("is_output", BindingType::Int),
            ],
            BindingType::Void,
            false,
        ),
        (
            "av_find_best_stream",
            vec![
                ("ic", fmt_ctx_ptr.clone()),
                ("type_", BindingType::Int),
                ("wanted_stream_nb", BindingType::Int),
                ("related_stream", BindingType::Int),
                ("decoder_ret", BindingType::ptr(codec_ptr.clone())),
                ("flags", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        // avcodec
        (
            "avcodec_alloc_context3",
            vec![("codec", codec_ptr.clone())],
            codec_ctx_ptr.clone(),
            false,
        ),
        (
            "avcodec_free_context",
            vec![("avctx", BindingType::ptr(codec_ctx_ptr.clone()))],
            BindingType::Void,
            false,
        ),
        (
            "avcodec_open2",
            vec![
                ("avctx", codec_ctx_ptr.clone()),
                ("codec", codec_ptr.clone()),
                ("options", dict_ptr_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_send_packet",
            vec![
                ("avctx", codec_ctx_ptr.clone()),
                (
                    "avpkt",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVPacket".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_receive_frame",
            vec![
                ("avctx", codec_ctx_ptr.clone()),
                ("frame", frame_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_send_frame",
            vec![
                ("avctx", codec_ctx_ptr.clone()),
                (
                    "frame",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVFrame".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_receive_packet",
            vec![("avctx", codec_ctx_ptr.clone()), ("avpkt", pkt_ptr.clone())],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_parameters_to_context",
            vec![
                ("codec", codec_ctx_ptr.clone()),
                ("par", codec_params_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "avcodec_find_decoder",
            vec![("id", BindingType::Int)],
            codec_ptr.clone(),
            false,
        ),
        (
            "avcodec_find_encoder",
            vec![("id", BindingType::Int)],
            codec_ptr.clone(),
            false,
        ),
        (
            "avcodec_find_decoder_by_name",
            vec![("name", const_char_ptr.clone())],
            codec_ptr.clone(),
            false,
        ),
        (
            "avcodec_find_encoder_by_name",
            vec![("name", const_char_ptr.clone())],
            codec_ptr.clone(),
            false,
        ),
        // avutil - packet/frame lifecycle
        ("av_packet_alloc", vec![], pkt_ptr.clone(), false),
        (
            "av_packet_free",
            vec![("pkt", BindingType::ptr(pkt_ptr.clone()))],
            BindingType::Void,
            false,
        ),
        (
            "av_packet_unref",
            vec![("pkt", pkt_ptr.clone())],
            BindingType::Void,
            false,
        ),
        ("av_frame_alloc", vec![], frame_ptr.clone(), false),
        (
            "av_frame_free",
            vec![("frame", BindingType::ptr(frame_ptr.clone()))],
            BindingType::Void,
            false,
        ),
        (
            "av_frame_unref",
            vec![("frame", frame_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "av_frame_get_buffer",
            vec![("frame", frame_ptr.clone()), ("align", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        // avutil - memory
        (
            "av_malloc",
            vec![("size", BindingType::ULong)],
            void_ptr.clone(),
            false,
        ),
        (
            "av_mallocz",
            vec![("size", BindingType::ULong)],
            void_ptr.clone(),
            false,
        ),
        (
            "av_realloc",
            vec![("ptr", void_ptr.clone()), ("size", BindingType::ULong)],
            void_ptr.clone(),
            false,
        ),
        (
            "av_free",
            vec![("ptr", void_ptr.clone())],
            BindingType::Void,
            false,
        ),
        (
            "av_freep",
            vec![("ptr", void_ptr.clone())],
            BindingType::Void,
            false,
        ),
        // avutil - misc
        (
            "av_log",
            vec![
                ("avcl", void_ptr.clone()),
                ("level", BindingType::Int),
                ("fmt", const_char_ptr.clone()),
            ],
            BindingType::Void,
            true,
        ),
        (
            "av_log_set_level",
            vec![("level", BindingType::Int)],
            BindingType::Void,
            false,
        ),
        ("av_version_info", vec![], const_char_ptr.clone(), false),
        ("avutil_version", vec![], BindingType::UInt, false),
        ("avcodec_version", vec![], BindingType::UInt, false),
        ("avformat_version", vec![], BindingType::UInt, false),
        (
            "av_get_pix_fmt_name",
            vec![("pix_fmt", BindingType::Int)],
            const_char_ptr.clone(),
            false,
        ),
        (
            "av_get_sample_fmt_name",
            vec![("sample_fmt", BindingType::Int)],
            const_char_ptr.clone(),
            false,
        ),
        (
            "av_rescale_q",
            vec![
                ("a", BindingType::LongLong),
                ("bq", BindingType::RecordRef("AVRational".into())),
                ("cq", BindingType::RecordRef("AVRational".into())),
            ],
            BindingType::LongLong,
            false,
        ),
        (
            "av_rescale_q_rnd",
            vec![
                ("a", BindingType::LongLong),
                ("bq", BindingType::RecordRef("AVRational".into())),
                ("cq", BindingType::RecordRef("AVRational".into())),
                ("rnd", BindingType::Int),
            ],
            BindingType::LongLong,
            false,
        ),
        // dict
        (
            "av_dict_get",
            vec![
                (
                    "m",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVDictionary".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("key", const_char_ptr.clone()),
                (
                    "prev",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("AVDictionaryEntry".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("flags", BindingType::Int),
            ],
            BindingType::ptr(BindingType::RecordRef("AVDictionaryEntry".into())),
            false,
        ),
        (
            "av_dict_set",
            vec![
                ("pm", dict_ptr_ptr.clone()),
                ("key", const_char_ptr.clone()),
                ("value", const_char_ptr.clone()),
                ("flags", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "av_dict_free",
            vec![("m", dict_ptr_ptr.clone())],
            BindingType::Void,
            false,
        ),
        // swscale
        (
            "sws_getContext",
            vec![
                ("srcW", BindingType::Int),
                ("srcH", BindingType::Int),
                ("srcFormat", BindingType::Int),
                ("dstW", BindingType::Int),
                ("dstH", BindingType::Int),
                ("dstFormat", BindingType::Int),
                ("flags", BindingType::Int),
                ("srcFilter", void_ptr.clone()),
                ("dstFilter", void_ptr.clone()),
                (
                    "param",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::Double),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
            ],
            BindingType::ptr(BindingType::RecordRef("SwsContext".into())),
            false,
        ),
        (
            "sws_scale",
            vec![
                (
                    "c",
                    BindingType::ptr(BindingType::RecordRef("SwsContext".into())),
                ),
                ("srcSlice", BindingType::ptr(const_uint8_ptr.clone())),
                (
                    "srcStride",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::Int),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("srcSliceY", BindingType::Int),
                ("srcSliceH", BindingType::Int),
                ("dst", BindingType::ptr(uint8_ptr.clone())),
                (
                    "dstStride",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::Int),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "sws_freeContext",
            vec![(
                "swsContext",
                BindingType::ptr(BindingType::RecordRef("SwsContext".into())),
            )],
            BindingType::Void,
            false,
        ),
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
        ("AVERROR_EOF", -541478725i128),
        ("AVERROR_INVALIDDATA", -1094995529),
        ("AV_LOG_QUIET", -8),
        ("AV_LOG_PANIC", 0),
        ("AV_LOG_FATAL", 8),
        ("AV_LOG_ERROR", 16),
        ("AV_LOG_WARNING", 24),
        ("AV_LOG_INFO", 32),
        ("AV_LOG_VERBOSE", 40),
        ("AV_LOG_DEBUG", 48),
        ("AV_LOG_TRACE", 56),
        ("AV_NOPTS_VALUE", -9223372036854775808),
        ("AV_TIME_BASE", 1000000),
        ("SWS_BILINEAR", 1),
        ("SWS_BICUBIC", 4),
        ("SWS_LANCZOS", 512),
        ("AVSEEK_FLAG_BACKWARD", 1),
        ("AVSEEK_FLAG_BYTE", 2),
        ("AVSEEK_FLAG_ANY", 4),
        ("AVSEEK_FLAG_FRAME", 8),
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

    // link surface
    for lib in ["avformat", "avcodec", "avutil", "swscale", "swresample"] {
        pkg.link.libraries.push(LinkLibrary {
            name: lib.into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
    }

    pkg
}
