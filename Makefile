.DEFAULT_GOAL := help
.SHELLFLAGS := -eu -o pipefail -c
SHELL := bash

# =============================================================================
# Language detection
# =============================================================================
# Each variable is non-empty if that language is present in the repo.

HAS_PYTHON := $(shell test -f pyproject.toml && echo 1)
HAS_PNPM   := $(shell test -f pnpm-lock.yaml && echo 1)
HAS_NPM    := $(shell test -f package-lock.json && echo 1)
HAS_NODE   := $(shell test -f package.json && echo 1)
HAS_RUST   := $(shell test -f Cargo.toml && echo 1)
HAS_TF     := $(shell find . -maxdepth 3 -name '*.tf' -not -path './.terraform/*' -print -quit 2>/dev/null)
HAS_DOCKER := $(shell test -f Dockerfile -o -f docker-compose.yml -o -f compose.yml && echo 1)
HAS_TILT   := $(shell test -f Tiltfile && echo 1)
HAS_TOOLVERSIONS := $(shell test -f .tool-versions && echo 1)

# Pick the JS package manager: pnpm preferred, fall back to npm
ifdef HAS_PNPM
JS_PM := pnpm
else ifdef HAS_NPM
JS_PM := npm
endif

# =============================================================================
# Help
# =============================================================================

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "; printf "\nUsage: make \033[36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2} /^##@/ {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Detected:"
	@$(if $(HAS_PYTHON),echo "  - Python (uv)",)
	@$(if $(JS_PM),echo "  - JavaScript/TypeScript ($(JS_PM))",)
	@$(if $(HAS_RUST),echo "  - Rust (cargo)",)
	@$(if $(HAS_TF),echo "  - Terraform",)
	@$(if $(HAS_DOCKER),echo "  - Docker",)
	@$(if $(HAS_TILT),echo "  - Tilt",)
	@echo ""

# =============================================================================
##@ Setup
# =============================================================================

.PHONY: bootstrap
bootstrap: ## Install dependencies and set up pre-commit hooks
	@echo "==> Bootstrapping project..."
	@test -f .env || (test -f .env.example && cp .env.example .env && echo "Created .env from .env.example")
ifdef HAS_PYTHON
	@echo "==> Installing Python dependencies (uv)..."
	@uv sync
endif
ifdef JS_PM
	@echo "==> Installing JS dependencies ($(JS_PM))..."
	@$(JS_PM) install
endif
ifdef HAS_RUST
	@echo "==> Fetching Rust dependencies..."
	@cargo fetch
endif
	@echo "==> Installing pre-commit hooks..."
	@command -v pre-commit >/dev/null 2>&1 && pre-commit install || echo "pre-commit not installed; skipping"
	@echo "==> Bootstrap complete."
ifdef HAS_TOOLVERSIONS
	@if command -v mise >/dev/null 2>&1; then \
		echo "==> Installing tool versions (mise)..."; \
		mise install; \
	elif command -v asdf >/dev/null 2>&1; then \
		echo "==> Installing tool versions (asdf)..."; \
		asdf install; \
	else \
		echo "==> .tool-versions present but neither mise nor asdf is installed."; \
		echo "    Install mise (https://mise.jdx.dev) for automatic version management,"; \
		echo "    or ensure your local toolchain matches the versions in .tool-versions."; \
	fi
endif

.PHONY: install
install: bootstrap ## Alias for bootstrap

# =============================================================================
##@ Development
# =============================================================================

.PHONY: dev
dev: ## Start project for local development
ifdef HAS_TILT
	@tilt up
else ifdef HAS_DOCKER
	@docker compose up
else
	@echo "No dev orchestrator detected (no Tiltfile or compose.yml)."
	@echo "Override 'dev' in a project-specific Makefile or define one."
	@exit 1
endif

.PHONY: dev-down
dev-down: ## Stop local development environment
ifdef HAS_TILT
	@tilt down
else ifdef HAS_DOCKER
	@docker compose down
endif

# =============================================================================
##@ Quality
# =============================================================================

.PHONY: format
format: ## Apply formatters across all detected languages
ifdef HAS_PYTHON
	@echo "==> Formatting Python..."
	@uv run ruff format .
	@uv run ruff check --fix .
endif
ifdef JS_PM
	@echo "==> Formatting JS/TS..."
	@$(JS_PM) run format || $(JS_PM) exec prettier --write .
endif
ifdef HAS_RUST
	@echo "==> Formatting Rust..."
	@cargo fmt --all
endif
ifdef HAS_TF
	@echo "==> Formatting Terraform..."
	@terraform fmt -recursive
endif

.PHONY: lint
lint: ## Run linters across all detected languages
ifdef HAS_PYTHON
	@echo "==> Linting Python..."
	@uv run ruff check .
	@uv run ruff format --check .
endif
ifdef JS_PM
	@echo "==> Linting JS/TS..."
	@$(JS_PM) run lint
endif
ifdef HAS_RUST
	@echo "==> Linting Rust..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo fmt --all -- --check
endif
ifdef HAS_TF
	@echo "==> Checking Terraform formatting..."
	@terraform fmt -check -recursive
endif

.PHONY: typecheck
typecheck: ## Run type checkers
ifdef HAS_PYTHON
	@echo "==> Type-checking Python..."
	@uv run pyright || uv run mypy .
endif
ifdef JS_PM
	@echo "==> Type-checking TypeScript..."
	@$(JS_PM) exec tsc --noEmit
endif

.PHONY: test
test: ## Run all tests
ifdef HAS_PYTHON
	@echo "==> Testing Python..."
	@uv run pytest
endif
ifdef JS_PM
	@echo "==> Testing JS/TS..."
	@$(JS_PM) test
endif
ifdef HAS_RUST
	@echo "==> Testing Rust..."
	@cargo test --all-features
endif

.PHONY: test-coverage
test-coverage: ## Run tests with coverage reporting
ifdef HAS_PYTHON
	@uv run pytest --cov --cov-report=term-missing --cov-report=xml
endif
ifdef JS_PM
	@$(JS_PM) test -- --coverage
endif
ifdef HAS_RUST
	@cargo tarpaulin --out Xml || echo "cargo-tarpaulin not installed; skipping Rust coverage"
endif

.PHONY: check
check: lint typecheck test ## Run full CI-equivalent checks locally
	@echo "==> All checks passed."

# =============================================================================
##@ Build
# =============================================================================

.PHONY: build
build: ## Build all detected languages
ifdef HAS_PYTHON
	@echo "==> Building Python..."
	@if grep -q '\[tool.uv.workspace\]' pyproject.toml; then \
		uv build --all-packages; \
	else \
		uv build; \
	fi
endif
ifdef JS_PM
	@echo "==> Building JS/TS..."
	@$(JS_PM) run build
endif
ifdef HAS_RUST
	@echo "==> Building Rust (release)..."
	@cargo build --release
endif

# =============================================================================
##@ Docker
# =============================================================================

# Override IMAGE and TAG as needed:
#   make docker-build IMAGE=ghcr.io/org/myproject TAG=v1.2.3
IMAGE ?= $(notdir $(CURDIR))
TAG   ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo "latest")

.PHONY: docker-build
docker-build: ## Build Docker image (override IMAGE and TAG vars)
	@docker build -t $(IMAGE):$(TAG) -t $(IMAGE):latest .

.PHONY: docker-run
docker-run: ## Run Docker image locally with .env
	@docker run --rm -it --env-file .env $(IMAGE):$(TAG)

.PHONY: docker-push
docker-push: ## Push Docker image
	@docker push $(IMAGE):$(TAG)
	@docker push $(IMAGE):latest

# =============================================================================
##@ Documentation
# =============================================================================

ADR_DIR     := docs/adrs
RUNBOOK_DIR := docs/runbooks

.PHONY: adr
adr: ## Create a new ADR (usage: make adr TITLE="Use Postgres")
	@if [ -z "$(TITLE)" ]; then \
		echo "ERROR: TITLE is required. Usage: make adr TITLE=\"Your decision title\""; \
		exit 1; \
	fi
	@if [ ! -f $(ADR_DIR)/template.md ]; then \
		echo "ERROR: $(ADR_DIR)/template.md not found."; \
		exit 1; \
	fi
	@next=$$(ls $(ADR_DIR) 2>/dev/null | grep -E '^[0-9]{4}-' | sort | tail -1 | grep -oE '^[0-9]{4}' || echo "0000"); \
	num=$$(printf "%04d" $$((10#$$next + 1))); \
	slug=$$(echo "$(TITLE)" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$$//'); \
	file="$(ADR_DIR)/$$num-$$slug.md"; \
	today=$$(date +%Y-%m-%d); \
	sed -e "s/ADR-NNNN: <short title of the decision>/ADR-$$num: $(TITLE)/" \
	    -e "s/<YYYY-MM-DD>/$$today/" \
	    $(ADR_DIR)/template.md > "$$file"; \
	echo "Created $$file"; \
	echo "Edit it, then add to the ADR index in $(ADR_DIR)/README.md"

.PHONY: runbook
runbook: ## Create a new runbook (usage: make runbook TITLE="Restore from backup")
	@if [ -z "$(TITLE)" ]; then \
		echo "ERROR: TITLE is required. Usage: make runbook TITLE=\"Your runbook title\""; \
		exit 1; \
	fi
	@if [ ! -f $(RUNBOOK_DIR)/template.md ]; then \
		echo "ERROR: $(RUNBOOK_DIR)/template.md not found."; \
		exit 1; \
	fi
	@slug=$$(echo "$(TITLE)" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$$//'); \
	file="$(RUNBOOK_DIR)/$$slug.md"; \
	if [ -e "$$file" ]; then \
		echo "ERROR: $$file already exists."; \
		exit 1; \
	fi; \
	today=$$(date +%Y-%m-%d); \
	sed -e "s/<short title — what this runbook is for>/$(TITLE)/" \
	    -e "s/<YYYY-MM-DD>/$$today/" \
	    $(RUNBOOK_DIR)/template.md > "$$file"; \
	echo "Created $$file"; \
	echo "Edit it, then add to the runbook index in $(RUNBOOK_DIR)/README.md"

.PHONY: docs-check
docs-check: ## Verify docs structure exists and templates are present
	@missing=0; \
	for f in docs/README.md $(ADR_DIR)/README.md $(ADR_DIR)/template.md $(RUNBOOK_DIR)/README.md $(RUNBOOK_DIR)/template.md; do \
		if [ ! -f "$$f" ]; then \
			echo "MISSING: $$f"; \
			missing=1; \
		fi; \
	done; \
	if [ $$missing -eq 0 ]; then \
		echo "==> Docs structure is complete."; \
	else \
		exit 1; \
	fi

.PHONY: adr-lint
adr-lint: ## Validate ADR integrity (numbering, status, supersession, index)
	@scripts/adr-lint.py --adr-dir $(ADR_DIR)

.PHONY: adr-lint-strict
adr-lint-strict: ## Validate ADRs with warnings promoted to errors (CI-equivalent)
	@scripts/adr-lint.py --adr-dir $(ADR_DIR) --strict

# =============================================================================
##@ Git
# =============================================================================

.PHONY: commit
commit: ## Stage changes (with confirmation), commit, and push
	@if git diff --quiet && git diff --staged --quiet && [ -z "$$(git ls-files --others --exclude-standard)" ]; then \
		echo "Nothing to commit. Working tree is clean."; \
		exit 0; \
	fi; \
	echo "==> Pending changes:"; \
	git status --short; \
	echo ""; \
	echo "==> Diff stat:"; \
	git diff --stat HEAD; \
	echo ""; \
	printf "Stage all changes (including untracked) and commit? [y/N] "; \
	read confirm; \
	case "$$confirm" in \
		y|Y|yes|YES) ;; \
		*) echo "Aborted."; exit 1 ;; \
	esac; \
	git add -A; \
	if command -v cz >/dev/null 2>&1; then \
		echo "==> Using commitizen (cz commit)"; \
		cz commit || exit 1; \
	elif command -v git-cz >/dev/null 2>&1; then \
		echo "==> Using commitizen (git cz)"; \
		git cz || exit 1; \
	else \
		echo "==> commitizen not found; opening git commit editor"; \
		echo "    (conventional-pre-commit hook will validate the message)"; \
		git commit || exit 1; \
	fi
	@if [ -z "$(NO_PUSH)" ]; then \
		$(MAKE) --no-print-directory push; \
	else \
		echo "==> NO_PUSH set; skipping push. Run 'make push' when ready."; \
	fi

.PHONY: push
push: ## Push current branch with safety checks
	@branch=$$(git rev-parse --abbrev-ref HEAD); \
	if [ "$$branch" = "HEAD" ]; then \
		echo "ERROR: detached HEAD state. Check out a branch before pushing."; \
		exit 1; \
	fi; \
	echo "==> Fetching origin/$$branch..."; \
	git fetch origin "$$branch" 2>/dev/null || true; \
	if [ "$$branch" = "main" ]; then \
		if git rev-parse --verify --quiet origin/main >/dev/null; then \
			local_sha=$$(git rev-parse main); \
			remote_sha=$$(git rev-parse origin/main); \
			base_sha=$$(git merge-base main origin/main); \
			if [ "$$local_sha" != "$$remote_sha" ]; then \
				if [ "$$base_sha" = "$$local_sha" ]; then \
					echo "ERROR: local main is behind origin/main."; \
					echo "       Run 'git pull --ff-only' before pushing."; \
					exit 1; \
				elif [ "$$base_sha" != "$$remote_sha" ]; then \
					echo "ERROR: local main has diverged from origin/main."; \
					echo "       Local has commits origin/main does not, AND vice versa."; \
					echo "       Resolve before pushing."; \
					exit 1; \
				fi; \
			fi; \
		fi; \
	fi; \
	echo "==> Pushing $$branch to origin..."; \
	git push --set-upstream origin "$$branch"

# =============================================================================
##@ Cleanup
# =============================================================================

.PHONY: clean
clean: ## Remove build artifacts (preserves dependencies)
	@echo "==> Cleaning build artifacts..."
	@rm -rf dist/ build/ out/ coverage/ .coverage htmlcov/ coverage.xml
	@rm -rf .pytest_cache/ .ruff_cache/ .mypy_cache/ .pyright/
	@rm -rf .turbo/ .next/ .nuxt/ .svelte-kit/ .vite/ .parcel-cache/ .tsbuildinfo
	@find . -type d -name __pycache__ -not -path './.git/*' -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name '*.pyc' -not -path './.git/*' -delete 2>/dev/null || true
ifdef HAS_RUST
	@cargo clean
endif
	@echo "==> Clean complete."

.PHONY: distclean
distclean: clean ## Remove everything (deps, venvs, lockfile caches)
	@echo "==> Removing dependencies and virtual environments..."
	@rm -rf node_modules/ .venv/ venv/ .yarn/ .pnpm-store/
	@rm -rf .terraform/ .terragrunt-cache/
	@rm -rf .tiltbuild/ tilt_modules/
	@echo "==> Distclean complete. Run 'make bootstrap' to rebuild."

# =============================================================================
##@ Info
# =============================================================================

.PHONY: detect
detect: ## Show what languages and tools are detected
	@echo "Python (pyproject.toml):     $(if $(HAS_PYTHON),yes,no)"
	@echo "JS package manager:          $(if $(JS_PM),$(JS_PM),none)"
	@echo "Rust (Cargo.toml):           $(if $(HAS_RUST),yes,no)"
	@echo "Terraform (*.tf):            $(if $(HAS_TF),yes,no)"
	@echo "Docker (Dockerfile/compose): $(if $(HAS_DOCKER),yes,no)"
	@echo "Tilt (Tiltfile):             $(if $(HAS_TILT),yes,no)"
	@echo "Tool versions (.tool-versions): $(if $(HAS_TOOLVERSIONS),yes,no)"
