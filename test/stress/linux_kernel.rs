use linc::ir::*;

/// Build a BindingPackage that mirrors a significant slice of Linux kernel
/// UAPI headers: ioctl, socket, netlink, input event, perf, etc.
///
/// The kernel UAPI surface is extreme: hundreds of structs, packed layouts,
/// bitfields, deeply nested unions, flexible array tails, and enormous enum
/// spaces for ioctl commands and socket options.
pub fn linux_kernel_package() -> BindingPackage {
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
    let char_ptr = BindingType::ptr(BindingType::Char);

    // --- basic POSIX typedefs ---
    for (name, target) in [
        ("size_t", BindingType::ULong),
        ("ssize_t", BindingType::Long),
        ("off_t", BindingType::LongLong),
        ("pid_t", BindingType::Int),
        ("uid_t", BindingType::UInt),
        ("gid_t", BindingType::UInt),
        ("mode_t", BindingType::UInt),
        ("dev_t", BindingType::ULong),
        ("ino_t", BindingType::ULong),
        ("nlink_t", BindingType::ULong),
        ("time_t", BindingType::LongLong),
        ("clockid_t", BindingType::Int),
        ("socklen_t", BindingType::UInt),
        ("sa_family_t", BindingType::UShort),
        ("in_port_t", BindingType::UShort),
        ("in_addr_t", BindingType::UInt),
        ("nfds_t", BindingType::ULong),
    ] {
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: name.into(),
            target,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
    }

    // --- socket structs ---
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("sockaddr".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("sa_family".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sa_data".into()),
                ty: BindingType::Array(Box::new(BindingType::Char), Some(14)),
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
        name: Some("in_addr".into()),
        fields: Some(vec![FieldBinding {
            name: Some("s_addr".into()),
            ty: BindingType::UInt,
            bit_width: None,
            layout: None,
        }]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("sockaddr_in".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("sin_family".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin_port".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin_addr".into()),
                ty: BindingType::RecordRef("in_addr".into()),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin_zero".into()),
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
        name: Some("in6_addr".into()),
        fields: Some(vec![FieldBinding {
            name: Some("s6_addr".into()),
            ty: BindingType::Array(Box::new(BindingType::UChar), Some(16)),
            bit_width: None,
            layout: None,
        }]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("sockaddr_in6".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("sin6_family".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin6_port".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin6_flowinfo".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin6_addr".into()),
                ty: BindingType::RecordRef("in6_addr".into()),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sin6_scope_id".into()),
                ty: BindingType::UInt,
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
        name: Some("sockaddr_un".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("sun_family".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sun_path".into()),
                ty: BindingType::Array(Box::new(BindingType::Char), Some(108)),
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // iovec
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("iovec".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("iov_base".into()),
                ty: void_ptr.clone(),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("iov_len".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // msghdr
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("msghdr".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("msg_name".into()),
                ty: void_ptr.clone(),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_namelen".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_iov".into()),
                ty: BindingType::ptr(BindingType::RecordRef("iovec".into())),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_iovlen".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_control".into()),
                ty: void_ptr.clone(),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_controllen".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("msg_flags".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // pollfd
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("pollfd".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("fd".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("events".into()),
                ty: BindingType::Short,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("revents".into()),
                ty: BindingType::Short,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // epoll_event — has bitfield in real life, we'll model it as a struct with
    // a bitfield to test the rejection path
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("epoll_event_packed".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("events".into()),
                ty: BindingType::UInt,
                bit_width: Some(32),
                layout: None,
            },
            FieldBinding {
                name: Some("data".into()),
                ty: BindingType::ULongLong,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // timespec / timeval
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("timespec".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("tv_sec".into()),
                ty: BindingType::LongLong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("tv_nsec".into()),
                ty: BindingType::Long,
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
        name: Some("timeval".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("tv_sec".into()),
                ty: BindingType::LongLong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("tv_usec".into()),
                ty: BindingType::Long,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // stat
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("stat".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("st_dev".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_ino".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_nlink".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_mode".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_uid".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_gid".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_rdev".into()),
                ty: BindingType::ULong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_size".into()),
                ty: BindingType::LongLong,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_blksize".into()),
                ty: BindingType::Long,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("st_blocks".into()),
                ty: BindingType::LongLong,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // input_event (linux/input.h)
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("input_event".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("time".into()),
                ty: BindingType::RecordRef("timeval".into()),
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("type_".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("code".into()),
                ty: BindingType::UShort,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("value".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // sigaction (simplified)
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("sigaction".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("sa_handler".into()),
                ty: BindingType::FunctionPointer {
                    return_type: Box::new(BindingType::Void),
                    parameters: vec![BindingType::Int],
                    variadic: false,
                },
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sa_flags".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("sa_mask".into()),
                ty: BindingType::Array(Box::new(BindingType::ULong), Some(16)),
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    // --- syscall-level functions ---
    let functions: Vec<(&str, Vec<(&str, BindingType)>, BindingType, bool)> = vec![
        // file ops
        (
            "open",
            vec![
                ("pathname", const_char_ptr.clone()),
                ("flags", BindingType::Int),
            ],
            BindingType::Int,
            true,
        ),
        (
            "close",
            vec![("fd", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "read",
            vec![
                ("fd", BindingType::Int),
                ("buf", void_ptr.clone()),
                ("count", BindingType::ULong),
            ],
            BindingType::Long,
            false,
        ),
        (
            "write",
            vec![
                ("fd", BindingType::Int),
                ("buf", const_void_ptr.clone()),
                ("count", BindingType::ULong),
            ],
            BindingType::Long,
            false,
        ),
        (
            "lseek",
            vec![
                ("fd", BindingType::Int),
                ("offset", BindingType::LongLong),
                ("whence", BindingType::Int),
            ],
            BindingType::LongLong,
            false,
        ),
        (
            "dup",
            vec![("oldfd", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "dup2",
            vec![("oldfd", BindingType::Int), ("newfd", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "pipe",
            vec![("pipefd", BindingType::ptr(BindingType::Int))],
            BindingType::Int,
            false,
        ),
        (
            "fcntl",
            vec![("fd", BindingType::Int), ("cmd", BindingType::Int)],
            BindingType::Int,
            true,
        ),
        (
            "ioctl",
            vec![("fd", BindingType::Int), ("request", BindingType::ULong)],
            BindingType::Int,
            true,
        ),
        (
            "stat",
            vec![
                ("pathname", const_char_ptr.clone()),
                (
                    "statbuf",
                    BindingType::ptr(BindingType::RecordRef("stat".into())),
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "fstat",
            vec![
                ("fd", BindingType::Int),
                (
                    "statbuf",
                    BindingType::ptr(BindingType::RecordRef("stat".into())),
                ),
            ],
            BindingType::Int,
            false,
        ),
        (
            "mmap",
            vec![
                ("addr", void_ptr.clone()),
                ("length", BindingType::ULong),
                ("prot", BindingType::Int),
                ("flags", BindingType::Int),
                ("fd", BindingType::Int),
                ("offset", BindingType::LongLong),
            ],
            void_ptr.clone(),
            false,
        ),
        (
            "munmap",
            vec![("addr", void_ptr.clone()), ("length", BindingType::ULong)],
            BindingType::Int,
            false,
        ),
        (
            "mprotect",
            vec![
                ("addr", void_ptr.clone()),
                ("len", BindingType::ULong),
                ("prot", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        // socket ops
        (
            "socket",
            vec![
                ("domain", BindingType::Int),
                ("type_", BindingType::Int),
                ("protocol", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "bind",
            vec![
                ("sockfd", BindingType::Int),
                (
                    "addr",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("sockaddr".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("addrlen", BindingType::UInt),
            ],
            BindingType::Int,
            false,
        ),
        (
            "listen",
            vec![("sockfd", BindingType::Int), ("backlog", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "accept",
            vec![
                ("sockfd", BindingType::Int),
                (
                    "addr",
                    BindingType::ptr(BindingType::RecordRef("sockaddr".into())),
                ),
                ("addrlen", BindingType::ptr(BindingType::UInt)),
            ],
            BindingType::Int,
            false,
        ),
        (
            "connect",
            vec![
                ("sockfd", BindingType::Int),
                (
                    "addr",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("sockaddr".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("addrlen", BindingType::UInt),
            ],
            BindingType::Int,
            false,
        ),
        (
            "send",
            vec![
                ("sockfd", BindingType::Int),
                ("buf", const_void_ptr.clone()),
                ("len", BindingType::ULong),
                ("flags", BindingType::Int),
            ],
            BindingType::Long,
            false,
        ),
        (
            "recv",
            vec![
                ("sockfd", BindingType::Int),
                ("buf", void_ptr.clone()),
                ("len", BindingType::ULong),
                ("flags", BindingType::Int),
            ],
            BindingType::Long,
            false,
        ),
        (
            "sendto",
            vec![
                ("sockfd", BindingType::Int),
                ("buf", const_void_ptr.clone()),
                ("len", BindingType::ULong),
                ("flags", BindingType::Int),
                (
                    "dest_addr",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("sockaddr".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("addrlen", BindingType::UInt),
            ],
            BindingType::Long,
            false,
        ),
        (
            "recvfrom",
            vec![
                ("sockfd", BindingType::Int),
                ("buf", void_ptr.clone()),
                ("len", BindingType::ULong),
                ("flags", BindingType::Int),
                (
                    "src_addr",
                    BindingType::ptr(BindingType::RecordRef("sockaddr".into())),
                ),
                ("addrlen", BindingType::ptr(BindingType::UInt)),
            ],
            BindingType::Long,
            false,
        ),
        (
            "sendmsg",
            vec![
                ("sockfd", BindingType::Int),
                (
                    "msg",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("msghdr".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                ("flags", BindingType::Int),
            ],
            BindingType::Long,
            false,
        ),
        (
            "recvmsg",
            vec![
                ("sockfd", BindingType::Int),
                (
                    "msg",
                    BindingType::ptr(BindingType::RecordRef("msghdr".into())),
                ),
                ("flags", BindingType::Int),
            ],
            BindingType::Long,
            false,
        ),
        (
            "setsockopt",
            vec![
                ("sockfd", BindingType::Int),
                ("level", BindingType::Int),
                ("optname", BindingType::Int),
                ("optval", const_void_ptr.clone()),
                ("optlen", BindingType::UInt),
            ],
            BindingType::Int,
            false,
        ),
        (
            "getsockopt",
            vec![
                ("sockfd", BindingType::Int),
                ("level", BindingType::Int),
                ("optname", BindingType::Int),
                ("optval", void_ptr.clone()),
                ("optlen", BindingType::ptr(BindingType::UInt)),
            ],
            BindingType::Int,
            false,
        ),
        (
            "shutdown",
            vec![("sockfd", BindingType::Int), ("how", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        // poll/epoll
        (
            "poll",
            vec![
                (
                    "fds",
                    BindingType::ptr(BindingType::RecordRef("pollfd".into())),
                ),
                ("nfds", BindingType::ULong),
                ("timeout", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "epoll_create1",
            vec![("flags", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        (
            "epoll_ctl",
            vec![
                ("epfd", BindingType::Int),
                ("op", BindingType::Int),
                ("fd", BindingType::Int),
                ("event", void_ptr.clone()),
            ],
            BindingType::Int,
            false,
        ),
        (
            "epoll_wait",
            vec![
                ("epfd", BindingType::Int),
                ("events", void_ptr.clone()),
                ("maxevents", BindingType::Int),
                ("timeout", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        // process
        ("fork", vec![], BindingType::Int, false),
        (
            "execve",
            vec![
                ("pathname", const_char_ptr.clone()),
                ("argv", BindingType::ptr(char_ptr.clone())),
                ("envp", BindingType::ptr(char_ptr.clone())),
            ],
            BindingType::Int,
            false,
        ),
        (
            "waitpid",
            vec![
                ("pid", BindingType::Int),
                ("wstatus", BindingType::ptr(BindingType::Int)),
                ("options", BindingType::Int),
            ],
            BindingType::Int,
            false,
        ),
        (
            "kill",
            vec![("pid", BindingType::Int), ("sig", BindingType::Int)],
            BindingType::Int,
            false,
        ),
        ("getpid", vec![], BindingType::Int, false),
        ("getppid", vec![], BindingType::Int, false),
        ("getuid", vec![], BindingType::UInt, false),
        ("getgid", vec![], BindingType::UInt, false),
        // signal
        (
            "sigaction",
            vec![
                ("signum", BindingType::Int),
                (
                    "act",
                    BindingType::Pointer {
                        pointee: Box::new(BindingType::RecordRef("sigaction".into())),
                        const_pointee: true,
                        qualifiers: TypeQualifiers::default(),
                    },
                ),
                (
                    "oldact",
                    BindingType::ptr(BindingType::RecordRef("sigaction".into())),
                ),
            ],
            BindingType::Int,
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
        ("O_RDONLY", 0i128),
        ("O_WRONLY", 1),
        ("O_RDWR", 2),
        ("O_CREAT", 64),
        ("O_EXCL", 128),
        ("O_TRUNC", 512),
        ("O_APPEND", 1024),
        ("O_NONBLOCK", 2048),
        ("SOCK_STREAM", 1),
        ("SOCK_DGRAM", 2),
        ("SOCK_RAW", 3),
        ("AF_UNIX", 1),
        ("AF_INET", 2),
        ("AF_INET6", 10),
        ("AF_NETLINK", 16),
        ("PROT_READ", 1),
        ("PROT_WRITE", 2),
        ("PROT_EXEC", 4),
        ("PROT_NONE", 0),
        ("MAP_SHARED", 1),
        ("MAP_PRIVATE", 2),
        ("MAP_ANONYMOUS", 32),
        ("POLLIN", 1),
        ("POLLOUT", 4),
        ("POLLERR", 8),
        ("POLLHUP", 16),
        ("EPOLL_CTL_ADD", 1),
        ("EPOLL_CTL_DEL", 2),
        ("EPOLL_CTL_MOD", 3),
        ("EPOLLIN", 1),
        ("EPOLLOUT", 4),
        ("EPOLLERR", 8),
        ("EPOLLHUP", 16),
        ("EPOLLET", 2147483648),
        ("SIGTERM", 15),
        ("SIGINT", 2),
        ("SIGKILL", 9),
        ("SIGUSR1", 10),
        ("STDIN_FILENO", 0),
        ("STDOUT_FILENO", 1),
        ("STDERR_FILENO", 2),
        ("F_GETFL", 3),
        ("F_SETFL", 4),
        ("F_GETFD", 1),
        ("F_SETFD", 2),
        ("SOL_SOCKET", 1),
        ("SO_REUSEADDR", 2),
        ("SO_KEEPALIVE", 9),
        ("SO_RCVBUF", 8),
        ("SO_SNDBUF", 7),
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

    // unsupported items (things gec should reject)
    pkg.items.push(BindingItem::Unsupported(UnsupportedItem {
        name: Some("__kernel_sigset_t".into()),
        reason: "platform-specific opaque kernel type".into(),
        source_offset: None,
    }));

    pkg
}
