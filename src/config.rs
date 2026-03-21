/// Configuration for a `gec` generation run.
///
/// Controls what gets generated and how.  Defaults produce a reasonable
/// binding crate with all supported item kinds enabled.
#[derive(Debug, Clone)]
pub struct GecConfig {
    /// Name for the generated crate (used in `Cargo.toml` and module docs).
    pub crate_name: String,
    /// Version string for the generated crate.
    pub crate_version: String,
    /// Whether to emit `extern "C"` function declarations.
    pub emit_functions: bool,
    /// Whether to emit `#[repr(C)]` record types.
    pub emit_records: bool,
    /// Whether to emit enum projections.
    pub emit_enums: bool,
    /// Whether to emit typedef aliases.
    pub emit_type_aliases: bool,
    /// Whether to emit global/static variable declarations.
    pub emit_variables: bool,
    /// Whether to emit Rust constants from `linc` macro bindings.
    pub emit_constants: bool,
    /// Whether to emit a `build.rs` with native link metadata.
    pub emit_build_script: bool,
}

impl Default for GecConfig {
    fn default() -> Self {
        Self {
            crate_name: "generated_bindings".into(),
            crate_version: "0.1.0".into(),
            emit_functions: true,
            emit_records: true,
            emit_enums: true,
            emit_type_aliases: true,
            emit_variables: true,
            emit_constants: true,
            emit_build_script: true,
        }
    }
}

impl GecConfig {
    pub fn new(crate_name: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enables_all() {
        let cfg = GecConfig::default();
        assert!(cfg.emit_functions);
        assert!(cfg.emit_records);
        assert!(cfg.emit_enums);
        assert!(cfg.emit_type_aliases);
        assert!(cfg.emit_variables);
        assert!(cfg.emit_constants);
        assert!(cfg.emit_build_script);
    }

    #[test]
    fn new_sets_crate_name() {
        let cfg = GecConfig::new("my_bindings");
        assert_eq!(cfg.crate_name, "my_bindings");
        assert!(cfg.emit_functions);
    }

    #[test]
    fn config_is_clone() {
        let cfg = GecConfig::default();
        let cfg2 = cfg.clone();
        assert_eq!(cfg.crate_name, cfg2.crate_name);
    }
}
