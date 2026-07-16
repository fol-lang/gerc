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

.PHONY: build b compile c fmt fmt-check lint check-features test t test-contract test-generated test-package test-system test-pipeline verify-pipeline docs-check verify help h clean docs release

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
	@tools/test-package.sh follang-gerc gerc

test-system:
	@bash tools/run-filtered-test.sh cargo test --test preservation_corpus -- --nocapture --test-threads=1

H5_PARC_REV := ba603cdccc9375473eca0c42e5462cf90b6da249
H5_LINC_REV := 4a2e0fba3aa528b7ba55626fd1ea9c75d153f7b1

test-pipeline:
	@test "$$(uname -s)" = Linux || { echo "H5 pipeline certification requires Linux"; exit 1; }
	@command -v gcc >/dev/null 2>&1 || { echo "gcc is required for H5 pipeline certification"; exit 1; }
	@command -v ar >/dev/null 2>&1 || { echo "ar is required for H5 pipeline certification"; exit 1; }
	@command -v rustc >/dev/null 2>&1 || { echo "rustc is required for H5 pipeline certification"; exit 1; }
	@set -eu; \
		gcc="$${GERC_H5_GCC:-$$(command -v gcc)}"; \
		ar="$${GERC_H5_AR:-$$(command -v ar)}"; \
		clang="$${GERC_H5_CLANG:-}"; \
		GERC_H5_RUN=1 GERC_H5_GCC="$$gcc" GERC_H5_AR="$$ar" GERC_H5_CLANG="$$clang" RUSTC="$$(command -v rustc)" \
			bash tools/run-filtered-test.sh cargo test --features pipeline-native --test h5_pipeline -- --nocapture --test-threads=1

verify-pipeline:
	@set -eu; \
		for sibling in parc linc; do \
			test -d "../$$sibling/.git" || { echo "../$$sibling must be an audited Git checkout"; exit 1; }; \
			test -z "$$(git -C "../$$sibling" status --porcelain=v1 --untracked-files=all)" || { echo "../$$sibling must be clean"; exit 1; }; \
		done; \
		test "$$(git -C ../parc rev-parse HEAD)" = "$(H5_PARC_REV)" || { echo "../parc is not at certified H5 revision $(H5_PARC_REV)"; exit 1; }; \
		test "$$(git -C ../linc rev-parse HEAD)" = "$(H5_LINC_REV)" || { echo "../linc is not at certified H5 revision $(H5_LINC_REV)"; exit 1; }; \
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
	@echo "  release      Create a new release (TYPE=patch|minor|major)"
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

TYPE ?= patch
HAS_REL := $(shell command -v git-rel 2>/dev/null)

release:
	@if [ -z "$(HAS_REL)" ]; then \
		echo "git-rel is not installed. Please install it first."; \
		exit 1; \
	fi
	@if [ -z "$(TYPE)" ]; then \
		echo "Release type not specified. Use 'make release TYPE=[patch|minor|major|m.m.p]'"; \
		exit 1; \
	fi
	@git rel $(TYPE)
