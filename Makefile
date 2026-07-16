PROJECT_NAME := $(shell sed -n 's/^name = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
PROJECT_CAP  := $(shell echo $(PROJECT_NAME) | tr '[:lower:]' '[:upper:]')
CURRENT_VERSION := $(shell grep '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
LATEST_TAG   ?= $(shell git describe --tags --abbrev=0 2>/dev/null)
TOP_DIR      := $(CURDIR)
BUILD_DIR    := $(TOP_DIR)/target

ifeq ($(PROJECT_NAME),)
$(error Error: project name not found in Cargo.toml)
endif

$(info ------------------------------------------)
$(info Project: $(PROJECT_NAME))
$(info Version: $(CURRENT_VERSION))
$(info ------------------------------------------)

.PHONY: build b compile c fmt fmt-check lint check-features test t test-contract test-generated test-package test-system test-pipeline verify-pipeline docs-check verify release-check help h clean docs

SHELL := /bin/bash


build:
	@cargo build --release

b: build

compile:
	@cargo clean
	@make build

c: compile

test:
	@cargo test -- --test-threads=1
	@cargo test --doc

t: test

fmt:
	@cargo fmt

fmt-check:
	@cargo fmt -- --check

lint:
	@cargo clippy --no-deps --all-targets --all-features -- -D warnings

check-features:
	@cargo check --all-targets
	@cargo check --all-targets --all-features
	@cargo check --all-targets --no-default-features

test-contract:
	@bash tools/run-filtered-test.sh cargo test --test contract_h1 -- --test-threads=1

test-generated:
	@set -eu; \
		gcc="$${GERC_H4_GCC:-gcc}"; \
		command -v "$$gcc" >/dev/null 2>&1 || { echo "$$gcc is required for the certified generated ABI lane"; exit 1; }; \
		command -v rustc >/dev/null 2>&1 || { echo "rustc is required for the certified generated ABI lane"; exit 1; }; \
		GERC_H4_GCC="$$(command -v "$$gcc")" bash tools/run-filtered-test.sh cargo test --test h4_native -- --nocapture --test-threads=1

test-package:
	@set -eu; \
		rustc="$$(if command -v rustup >/dev/null 2>&1; then rustup which rustc; else command -v rustc; fi)"; \
		test -x "$$rustc" || { echo "rustc is required for the package pipeline"; exit 1; }; \
		PATH="$$(dirname "$$rustc"):$$PATH" RUSTC="$$rustc" \
		GERC_PARC_RELEASE_REVISION=$(PARC_RELEASE_REVISION) \
		GERC_LINC_RELEASE_REVISION=$(LINC_RELEASE_REVISION) \
		tools/test-package.sh follang-gerc gerc

test-system:
	@bash tools/run-filtered-test.sh cargo test --test preservation_corpus -- --nocapture --test-threads=1

PARC_RELEASE_REVISION := 0f52aeeeeec47a082c0d8a515130ee853aa1101d
LINC_RELEASE_REVISION := c874d5b0332249524422d9d08c35b3d4edd7e3fa

test-pipeline:
	@test "$$(uname -s)" = Linux || { echo "H5 pipeline certification requires Linux"; exit 1; }
	@command -v gcc >/dev/null 2>&1 || { echo "gcc is required for H5 pipeline certification"; exit 1; }
	@command -v ar >/dev/null 2>&1 || { echo "ar is required for H5 pipeline certification"; exit 1; }
	@set -eu; \
		gcc="$${GERC_H5_GCC:-$$(command -v gcc)}"; \
		ar="$${GERC_H5_AR:-$$(command -v ar)}"; \
		clang="$${GERC_H5_CLANG:-}"; \
		rustc="$$(if command -v rustup >/dev/null 2>&1; then rustup which rustc; else command -v rustc; fi)"; \
		test -x "$$rustc" || { echo "rustc is required for H5 pipeline certification"; exit 1; }; \
		GERC_H5_RUN=1 GERC_H5_GCC="$$gcc" GERC_H5_AR="$$ar" GERC_H5_CLANG="$$clang" RUSTC="$$rustc" \
			bash tools/run-filtered-test.sh cargo test --features pipeline-native --test h5_pipeline -- --nocapture --test-threads=1

verify-pipeline:
	@set -eu; \
		for sibling in parc linc; do \
			test -d "../$$sibling/.git" || { echo "../$$sibling must be an audited Git checkout"; exit 1; }; \
			test -z "$$(git -C "../$$sibling" status --porcelain=v1 --untracked-files=all)" || { echo "../$$sibling must be clean"; exit 1; }; \
		done; \
		test "$$(git -C ../parc rev-parse HEAD)" = "$(PARC_RELEASE_REVISION)" || { echo "../parc is not at required revision $(PARC_RELEASE_REVISION)"; exit 1; }; \
		test "$$(git -C ../linc rev-parse HEAD)" = "$(LINC_RELEASE_REVISION)" || { echo "../linc is not at required revision $(LINC_RELEASE_REVISION)"; exit 1; }; \
		$(MAKE) -C ../parc build; \
		$(MAKE) -C ../linc check-features
	@$(MAKE) test-pipeline

docs-check:
	@command -v mdbook >/dev/null 2>&1 || { echo "mdbook is required"; exit 1; }
	@mdbook build $(TOP_DIR)/book --dest-dir $(BUILD_DIR)/book
	@cargo doc --no-deps --all-features

VERIFY_ALLOW_DIRTY ?= 0

verify:
	@set -eu; \
		before="$$(mktemp "$${TMPDIR:-/tmp}/gerc-verify-before.XXXXXX")"; \
		after="$$(mktemp "$${TMPDIR:-/tmp}/gerc-verify-after.XXXXXX")"; \
		trap 'rm -f "$$before" "$$after"' EXIT; \
		git status --porcelain=v1 --untracked-files=all >"$$before"; \
		if test -s "$$before" && test "$(VERIFY_ALLOW_DIRTY)" != 1; then \
			echo "verification requires a clean worktree (or VERIFY_ALLOW_DIRTY=1)"; \
			cat "$$before"; \
			exit 1; \
		fi; \
		$(MAKE) fmt-check; \
		$(MAKE) lint; \
		$(MAKE) check-features; \
		$(MAKE) test; \
		$(MAKE) test-contract; \
		$(MAKE) test-generated; \
		$(MAKE) verify-pipeline; \
		$(MAKE) test-package; \
		$(MAKE) test-system; \
		$(MAKE) docs-check; \
		git status --porcelain=v1 --untracked-files=all >"$$after"; \
		diff -u "$$before" "$$after"

help:
	@echo
	@echo "Usage: make [target]"
	@echo
	@echo "Available targets:"
	@echo "  build        Build project"
	@echo "  compile      Configure and generate build files"
	@echo "  fmt          Format this package"
	@echo "  fmt-check    Check Rust formatting"
	@echo "  lint         Run Clippy with warnings denied"
	@echo "  check-features  Check default, all, and no-default features"
	@echo "  test         Run tests"
	@echo "  test-contract  Run contract tests"
	@echo "  test-generated Run the explicit GCC generated C/Rust ABI lane"
	@echo "  test-package   Test the package archive and clean consumer"
	@echo "  test-system    Run required system tests"
	@echo "  test-pipeline  Run the H5 production-API corpus"
	@echo "  verify-pipeline Verify exact clean siblings and the full H5 corpus"
	@echo "  docs-check     Build Rust and mdBook documentation"
	@echo "  verify         Run the complete non-mutating gate"
	@echo "  docs         Build documentation (TYPE=mdbook|doxygen)"
	@echo "  release-check  Verify clean, synchronized release eligibility"
	@echo

h : help

clean:
	@echo "Cleaning build directory..."
	@rm -rf $(BUILD_DIR)
	@echo "Build directory cleaned."

docs:
ifeq ($(TYPE),mdbook)
	@$(MAKE) docs-check
else ifeq ($(TYPE),doxygen)
	@command -v doxygen >/dev/null 2>&1 || { echo "doxygen is not installed. Please install it first."; exit 1; }
else
	$(error Invalid documentation type. Use 'make docs TYPE=mdbook' or 'make docs TYPE=doxygen')
endif

release-check:
	@set -eu; \
		branch="$$(git symbolic-ref --quiet --short HEAD)" || { \
			echo "release check requires a branch checkout, not detached HEAD"; \
			exit 1; \
		}; \
		upstream="$$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)" || { \
			echo "release check requires an upstream for $$branch"; \
			exit 1; \
		}; \
		test -z "$$(git status --porcelain=v1 --untracked-files=all)" || { \
			echo "release check requires a clean GERC worktree"; \
			exit 1; \
		}; \
		head="$$(git rev-parse HEAD)"; \
		upstream_head="$$(git rev-parse "$$upstream")"; \
		test "$$head" = "$$upstream_head" || { \
			echo "release check requires HEAD to equal $$upstream"; \
			echo "HEAD:     $$head"; \
			echo "upstream: $$upstream_head"; \
			exit 1; \
		}; \
		tag="follang-gerc-v$(CURRENT_VERSION)"; \
		! git rev-parse --quiet --verify "refs/tags/$$tag" >/dev/null || { \
			echo "release tag already exists: $$tag"; \
			exit 1; \
		}; \
		grep -Fqx 'publish = false' Cargo.toml || { \
			echo "registry publication must remain disabled"; \
			exit 1; \
		}; \
		for sibling in parc linc; do \
			case "$$sibling" in \
				parc) expected="$(PARC_RELEASE_REVISION)" ;; \
				linc) expected="$(LINC_RELEASE_REVISION)" ;; \
			esac; \
			sibling_path="$$(cd "../$$sibling" && pwd -P)"; \
			test -z "$$(git -C "$$sibling_path" status --porcelain=v1 --untracked-files=all)" || { \
				echo "release check requires a clean $$sibling worktree"; \
				exit 1; \
			}; \
			sibling_head="$$(git -C "$$sibling_path" rev-parse HEAD)"; \
			test "$$sibling_head" = "$$expected" || { \
				echo "release check requires $$sibling $$expected"; \
				echo "$$sibling: $$sibling_head"; \
				exit 1; \
			}; \
			sibling_branch="$$(git -C "$$sibling_path" symbolic-ref --quiet --short HEAD)" || { \
				echo "release check requires a $$sibling branch checkout"; \
				exit 1; \
			}; \
			sibling_upstream="$$(git -C "$$sibling_path" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)" || { \
				echo "release check requires an upstream for $$sibling $$sibling_branch"; \
				exit 1; \
			}; \
			sibling_upstream_head="$$(git -C "$$sibling_path" rev-parse "$$sibling_upstream")"; \
			test "$$sibling_head" = "$$sibling_upstream_head" || { \
				echo "release check requires $$sibling HEAD to equal $$sibling_upstream"; \
				echo "$$sibling HEAD:     $$sibling_head"; \
				echo "$$sibling upstream: $$sibling_upstream_head"; \
				exit 1; \
			}; \
		done; \
		$(MAKE) verify; \
		echo "release candidate is eligible: $$tag at $$head"; \
		echo "required PARC revision: $(PARC_RELEASE_REVISION)"; \
		echo "required LINC revision: $(LINC_RELEASE_REVISION)"; \
		echo "release-check is non-mutating; follow RELEASE.md to create an archive/tag"
