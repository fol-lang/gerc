#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificationState {
    SupportedAndTested,
    ExplicitlyRejected,
    ExperimentalNotForFol,
}

impl CertificationState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SupportedAndTested => "supported-and-tested",
            Self::ExplicitlyRejected => "explicitly-rejected",
            Self::ExperimentalNotForFol => "experimental-not-for-FOL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixRow {
    pub construct: &'static str,
    pub state: CertificationState,
    pub owner: &'static str,
    pub case: &'static str,
    pub stable_code: Option<&'static str>,
}

pub const TYPE_MATRIX: &[MatrixRow] = &[
    supported("void", "positive-abi-roundtrip"),
    supported("_Bool", "positive-abi-roundtrip"),
    supported("plain char", "positive-abi-roundtrip"),
    supported("signed char", "positive-abi-roundtrip"),
    supported("unsigned char", "positive-abi-roundtrip"),
    supported("signed short", "positive-abi-roundtrip"),
    supported("unsigned short", "positive-abi-roundtrip"),
    supported("signed int", "positive-abi-roundtrip"),
    supported("unsigned int", "positive-abi-roundtrip"),
    supported("signed long", "positive-abi-roundtrip"),
    supported("unsigned long", "positive-abi-roundtrip"),
    supported("signed long long", "positive-abi-roundtrip"),
    supported("unsigned long long", "positive-abi-roundtrip"),
    supported("float", "positive-abi-roundtrip"),
    supported("double", "positive-abi-roundtrip"),
    supported("raw and nullable pointers", "positive-abi-roundtrip"),
    supported("nonzero fixed arrays", "positive-abi-roundtrip"),
    supported(
        "complete records with measured layout",
        "positive-abi-roundtrip",
    ),
    supported(
        "complete unions with measured layout",
        "positive-abi-roundtrip",
    ),
    supported(
        "incomplete records behind pointers",
        "positive-abi-roundtrip",
    ),
    supported(
        "C enums as integer aliases and constants",
        "positive-abi-roundtrip",
    ),
    supported("C calling-convention routines", "positive-abi-roundtrip"),
    supported("C calling-convention callbacks", "positive-abi-roundtrip"),
    rejected(
        "long double and extended floating types",
        "LINC",
        "reject-long-double",
        "LINC-E3014",
    ),
    rejected(
        "complex floating types",
        "LINC",
        "reject-complex",
        "LINC-E3014",
    ),
    rejected(
        "compiler vector types",
        "PARC",
        "reject-vector-closure",
        "CompletionBlocker::Unsupported",
    ),
    rejected("_BitInt", "PARC", "reject-bit-int", "PARC-P1107"),
    rejected("128-bit C integers", "LINC", "reject-int128", "LINC-E3014"),
    rejected(
        "by-value opaque or incomplete records",
        "PARC",
        "reject-opaque-by-value",
        "CompletionBlocker::IncompleteRecord",
    ),
    rejected("bitfield layouts", "GERC", "reject-bitfields", "GERC-E2002"),
    rejected(
        "unsupported calling conventions",
        "LINC",
        "reject-ms-abi-on-linux",
        "LINC-E3050",
    ),
    rejected(
        "variadic or unspecified callables",
        "LINC",
        "reject-variadic",
        "LINC-E3050",
    ),
    rejected(
        "C++ types and ABI",
        "PARC",
        "reject-cpp-source",
        "PARC-P0002",
    ),
    rejected("thread-local globals", "GERC", "reject-tls", "GERC-E2002"),
    experimental(
        "function-like macros",
        "GERC",
        "preserve-nonemitted-macros",
        "GERC-N3000",
    ),
    experimental(
        "string macros",
        "GERC",
        "preserve-nonemitted-macros",
        "GERC-N3000",
    ),
];

pub const PLATFORM_MATRIX: &[MatrixRow] = &[
    supported_platform(
        "x86_64-unknown-linux-gnu / ELF / explicit GCC",
        "positive-abi-roundtrip",
    ),
    experimental(
        "x86_64-unknown-linux-gnu / ELF / Clang differential",
        "H5 differential lane",
        "clang-differential",
        "optional full typed value roundtrip",
    ),
    rejected(
        "x86_64-unknown-linux-musl",
        "H5 gate",
        "reject-uncertified-platforms",
        "not certified",
    ),
    rejected(
        "second Linux architecture",
        "H5 gate",
        "reject-uncertified-platforms",
        "not certified",
    ),
    rejected(
        "aarch64-apple-darwin",
        "H5 gate",
        "reject-uncertified-platforms",
        "not certified",
    ),
    rejected(
        "x86_64-pc-windows-msvc and MinGW",
        "H5 gate",
        "reject-uncertified-platforms",
        "not certified",
    ),
];

pub const PROVIDER_FAILURES: &[MatrixRow] = &[
    rejected(
        "missing provider symbol",
        "LINC",
        "reject-missing",
        "LINC-E3040",
    ),
    rejected(
        "hidden provider symbol",
        "LINC",
        "reject-hidden",
        "LINC-E3040",
    ),
    rejected("weak provider symbol", "LINC", "reject-weak", "LINC-E3040"),
    rejected(
        "duplicate symbols in one provider",
        "LINC",
        "reject-duplicate",
        "LINC-E3040",
    ),
    rejected(
        "ambiguous symbols across providers",
        "LINC",
        "reject-ambiguous",
        "LINC-E3040",
    ),
    rejected(
        "wrong-target provider",
        "LINC",
        "reject-wrong-target",
        "LINC-E3007",
    ),
    rejected(
        "partial external-preprocessor source",
        "PARC",
        "reject-partial-source",
        "PARC-P0001",
    ),
    rejected(
        "stale source-bound evidence",
        "GERC",
        "reject-stale-evidence",
        "GERC-E1100",
    ),
];

pub const REQUIRED_RUNTIME_CASES: &[&str] = &[
    "positive-abi-roundtrip",
    "preserve-nonemitted-macros",
    "reject-long-double",
    "reject-complex",
    "reject-vector-closure",
    "reject-bit-int",
    "reject-int128",
    "reject-opaque-by-value",
    "reject-bitfields",
    "reject-ms-abi-on-linux",
    "reject-variadic",
    "reject-cpp-source",
    "reject-tls",
    "reject-uncertified-platforms",
    "reject-missing",
    "reject-hidden",
    "reject-weak",
    "reject-duplicate",
    "reject-ambiguous",
    "reject-wrong-target",
    "reject-partial-source",
    "reject-stale-evidence",
];

const fn supported(construct: &'static str, case: &'static str) -> MatrixRow {
    MatrixRow {
        construct,
        state: CertificationState::SupportedAndTested,
        owner: "PARC/LINC/GERC",
        case,
        stable_code: None,
    }
}

const fn supported_platform(construct: &'static str, case: &'static str) -> MatrixRow {
    MatrixRow {
        construct,
        state: CertificationState::SupportedAndTested,
        owner: "H5 pipeline",
        case,
        stable_code: None,
    }
}

const fn rejected(
    construct: &'static str,
    owner: &'static str,
    case: &'static str,
    stable_code: &'static str,
) -> MatrixRow {
    MatrixRow {
        construct,
        state: CertificationState::ExplicitlyRejected,
        owner,
        case,
        stable_code: Some(stable_code),
    }
}

const fn experimental(
    construct: &'static str,
    owner: &'static str,
    case: &'static str,
    stable_code: &'static str,
) -> MatrixRow {
    MatrixRow {
        construct,
        state: CertificationState::ExperimentalNotForFol,
        owner,
        case,
        stable_code: Some(stable_code),
    }
}
