#!/usr/bin/env bash
set -euo pipefail

package_name=${1:?package name is required}
crate_name=${2:?crate name is required}
expected_parc_revision=${GERC_PARC_RELEASE_REVISION:?PARC release revision is required}
expected_linc_revision=${GERC_LINC_RELEASE_REVISION:?LINC release revision is required}
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
parc_checkout=$(cd "$root/../parc" && pwd -P)
linc_checkout=$(cd "$root/../linc" && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/${crate_name}-package.XXXXXX")
trap 'rm -rf "$scratch"' EXIT

for sibling in parc linc; do
    case "$sibling" in
        parc)
            checkout=$parc_checkout
            expected=$expected_parc_revision
            ;;
        linc)
            checkout=$linc_checkout
            expected=$expected_linc_revision
            ;;
    esac
    revision=$(git -C "$checkout" rev-parse HEAD)
    if test "$revision" != "$expected"; then
        echo "$sibling revision mismatch: expected $expected, found $revision" >&2
        exit 1
    fi
    if test -n "$(git -C "$checkout" status --porcelain=v1 --untracked-files=all)"; then
        echo "$sibling package input must be a clean worktree" >&2
        exit 1
    fi
done

copy_source() {
    local source_root=$1
    local destination=$2

    mkdir -p "$destination"
    tar --exclude='./.git' --exclude='./target' -cf - -C "$source_root" . \
        | tar -xf - -C "$destination"
}

mkdir -p "$scratch/source"
copy_source "$parc_checkout" "$scratch/source/parc"
copy_source "$linc_checkout" "$scratch/source/linc"
copy_source "$root" "$scratch/source/gerc"
parc_root="$scratch/source/parc"
linc_root="$scratch/source/linc"
root="$scratch/source/gerc"

package_and_extract() {
    local source_root=$1
    local distribution=$2
    local cargo_config=${3:-}
    local version
    local archive
    local cargo_args=(package --manifest-path "$source_root/Cargo.toml" --allow-dirty --no-verify)

    version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$source_root/Cargo.toml" | head -n 1)
    archive="$source_root/target/package/${distribution}-${version}.crate"
    rm -f "$archive"
    if test -n "$cargo_config"; then
        cargo_args+=(--config "$cargo_config")
    fi
    cargo "${cargo_args[@]}" >&2
    test -f "$archive"
    mkdir -p "$scratch/packages"
    tar -xzf "$archive" -C "$scratch/packages"
    printf '%s' "$scratch/packages/${distribution}-${version}"
}

reject_dependency_paths() {
    local manifest=$1

    if awk '
        /^\[(target\..*\.)?(dependencies|dev-dependencies|build-dependencies)(\.|])/{ dependency = 1; next }
        /^\[/{ dependency = 0 }
        dependency && /^[[:space:]]*path[[:space:]]*=/{ found = 1 }
        END { exit found ? 0 : 1 }
    ' "$manifest"; then
        echo "normalized packaged manifest contains a path dependency: $manifest" >&2
        exit 1
    fi
}

parc_package=$(package_and_extract "$parc_root" follang-parc)
cat >"$scratch/source-patches.toml" <<EOF
[patch.crates-io]
follang-parc = { path = "$parc_root" }
EOF
linc_package=$(package_and_extract "$linc_root" follang-linc "$scratch/source-patches.toml")
cat >>"$scratch/source-patches.toml" <<EOF
follang-linc = { path = "$linc_root" }
EOF
gerc_package=$(package_and_extract "$root" "$package_name" "$scratch/source-patches.toml")
gerc_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$root/Cargo.toml" | head -n 1)
parc_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$parc_package/Cargo.toml" | head -n 1)
linc_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$linc_package/Cargo.toml" | head -n 1)

for package_dir in "$parc_package" "$linc_package" "$gerc_package"; do
    for file in README.md RELEASE.md LICENSE-MIT LICENSE-APACHE; do
        test -f "$package_dir/$file"
    done
    reject_dependency_paths "$package_dir/Cargo.toml"
    grep -Fqx 'publish = false' "$package_dir/Cargo.toml.orig"
    grep -Fqx 'license = "MIT OR Apache-2.0"' "$package_dir/Cargo.toml.orig"
    grep -Fqx 'rust-version = "1.89"' "$package_dir/Cargo.toml.orig"
done

grep -Fqx 'name = "follang-parc"' "$parc_package/Cargo.toml.orig"
grep -Fqx 'version = "0.16.0"' "$parc_package/Cargo.toml.orig"
grep -Fqx 'name = "follang-linc"' "$linc_package/Cargo.toml.orig"
grep -Fqx 'version = "0.1.0"' "$linc_package/Cargo.toml.orig"
grep -Fqx 'name = "follang-gerc"' "$gerc_package/Cargo.toml.orig"
grep -Fqx 'version = "0.1.0"' "$gerc_package/Cargo.toml.orig"
grep -Fqx 'name = "gerc"' "$gerc_package/Cargo.toml.orig"
for omitted in .github book flake.nix flake.lock Makefile tools; do
    test ! -e "$gerc_package/$omitted"
done
grep -Fq 'linc = { package = "follang-linc", version = "=0.1.0", path = "../linc", default-features = false, features = ["contracts"] }' \
    "$gerc_package/Cargo.toml.orig"
grep -Fq 'parc = { package = "follang-parc", version = "=0.16.0", path = "../parc" }' \
    "$gerc_package/Cargo.toml.orig"
grep -Fqx 'version = "=0.1.0"' "$gerc_package/Cargo.toml"
grep -Fqx 'package = "follang-linc"' "$gerc_package/Cargo.toml"
grep -Fqx 'version = "=0.16.0"' "$gerc_package/Cargo.toml"
grep -Fqx 'package = "follang-parc"' "$gerc_package/Cargo.toml"

mkdir -p "$scratch/consumer/src"
cat >"$scratch/Cargo.toml" <<EOF
[workspace]
resolver = "2"
members = [
    "packages/$(basename "$parc_package")",
    "packages/$(basename "$linc_package")",
    "packages/$(basename "$gerc_package")",
    "consumer",
]

[patch.crates-io]
follang-parc = { path = "$parc_package" }
follang-linc = { path = "$linc_package" }
follang-gerc = { path = "$gerc_package" }
EOF
cat >"$scratch/consumer/Cargo.toml" <<EOF
[package]
name = "${crate_name}-package-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
${crate_name} = { package = "${package_name}", version = "=${gerc_version}" }
parc = { package = "follang-parc", version = "=${parc_version}" }
linc = { package = "follang-linc", version = "=${linc_version}", default-features = false, features = ["contracts"] }
EOF
cat >"$scratch/consumer/src/lib.rs" <<EOF
use ${crate_name}::{GenerationBundle, GenerationError, GenerationRequest};

pub const PACKAGED_SCHEMA_ID: &str = ${crate_name}::GENERATION_SCHEMA_ID;
pub const PACKAGED_SCHEMA_VERSION: u16 = ${crate_name}::GENERATION_SCHEMA_VERSION;
pub const PACKAGED_ALGORITHM_ID: &str = ${crate_name}::GENERATION_ALGORITHM_ID;
pub const PACKAGED_GENERATOR_IDENTITY: &str = ${crate_name}::GENERATOR_IDENTITY;

pub fn generate_checked(
    request: GenerationRequest<'_>,
) -> Result<GenerationBundle, GenerationError> {
    ${crate_name}::generate(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_typed_pipeline_generates_and_checks_nonempty_output() {
        let package = parc::contract::decode_source_package(
            parc::contract::corpus::COMPLETE_SOURCE_PACKAGE_JSON,
        )
        .expect("packaged PARC corpus");
        let source = package
            .into_complete(&linc::contract::corpus::preservation_selection())
            .expect("complete packaged source");
        let evidence = linc::contract::corpus::validated_preservation_link_analysis(&source)
            .expect("validated packaged evidence");
        let id = |name: &str| {
            source
                .source()
                .declarations()
                .iter()
                .find(|declaration| {
                    declaration
                        .name
                        .as_ref()
                        .is_some_and(|source_name| source_name.normalized == name)
                })
                .expect("packaged declaration")
                .id
        };
        let selection = ${crate_name}::ItemSelection::try_new([
            id("parc_packet"),
            id("parc_mode"),
        ])
        .expect("typed packaged selection");
        let bundle = generate_checked(
            ${crate_name}::GenerationRequest::try_new(&source, &evidence, &selection)
                .expect("typed packaged request"),
        )
        .expect("packaged strict generation");
        let generated = bundle
            .files()
            .get("src/lib.rs")
            .and_then(|file| file.utf8_contents())
            .expect("nonempty packaged generated file");
        assert!(generated.contains("#![no_std]"));
        assert!(generated.contains("core::ffi::c_int"));
        assert!(!bundle.link_plan().atoms().is_empty());
    }
}
EOF

cargo generate-lockfile --manifest-path "$scratch/Cargo.toml" --offline
cargo test --manifest-path "$scratch/Cargo.toml" --workspace --offline --locked -- --test-threads=1
if test "$(uname -s)" = Linux; then
    command -v gcc >/dev/null 2>&1 || { echo "packaged pipeline requires gcc"; exit 1; }
    command -v ar >/dev/null 2>&1 || { echo "packaged pipeline requires ar"; exit 1; }
    command -v rustc >/dev/null 2>&1 || { echo "packaged pipeline requires rustc"; exit 1; }
    GERC_H5_RUN=1 \
        GERC_H5_GCC="$(command -v gcc)" \
        GERC_H5_AR="$(command -v ar)" \
        GERC_H5_CLANG="$(command -v clang 2>/dev/null || true)" \
        RUSTC="$(command -v rustc)" \
        cargo test --manifest-path "$scratch/Cargo.toml" -p "$package_name" \
            --features pipeline-native --test h5_pipeline --offline --locked -- \
            --nocapture --test-threads=1
fi
echo "packaged PARC/LINC/GERC archive workspace and typed consumer passed"
