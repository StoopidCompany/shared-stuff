# stoopid-commons

Generic, reusable infrastructure for polyglot microservice systems. Open source under MIT license.

## Overview

`stoopid-commons` is a collection of language-spanning utilities and patterns developed at Stoopid Company for building polyglot microservice architectures. It contains:

- **Per-language structured logging packages** — Python (structlog), TypeScript (pino), Rust (tracing). All produce JSON output with a common schema (`timestamp`, `level`, `service`, `correlation_id`, `span_id`, `message`, plus structured fields), so logs from any service in any language can be aggregated and queried uniformly.
- **Helm chart base** — a parameterized base chart that services extend with their own `values.yaml`. Captures common k8s deployment patterns (deployments, services, config maps, ingress, HPA, OTel sidecar) so individual services don't re-author manifests from scratch.
- **OpenTelemetry conventions** — span naming, attribute conventions, and helper packages per language for consistent instrumentation across services.
- **Common utilities** — correlation ID generation and propagation, retry policies with backoff, circuit breaker patterns, and other generic helpers per language.
- **Build tooling** — shared Makefile patterns, pre-commit configurations, and CI workflow templates that any cell or service can extend.

This repo is consumed as published packages, not as a source dependency. Other projects pull these via npm, PyPI, crates.io, and Helm chart repositories.

It does not contain anything specific to any particular product or architecture. Project-specific contracts and types live in their own repos.

## Setup

You only need to set up `stoopid-commons` locally if you're contributing to it. To use the published packages in your own projects, follow the consumer instructions in the **Usage** section below.

This repo is polyglot — Python, TypeScript, and Rust toolchains are all needed to build and publish all packages. Contributors working on only one language can install only that language's tooling.

### macOS

Install Homebrew if not present, then:

```bash
brew install git make
brew install python@3.12
brew install node
brew install rustup-init && rustup-init -y
brew install kubectl helm
brew install minikube
brew install pre-commit
```

After Rust install, restart your shell or `source "$HOME/.cargo/env"`.

Clone and bootstrap:

```bash
git clone https://github.com/stoopidco/stoopid-commons.git
cd stoopid-commons
make setup
```

### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y git make build-essential curl

# Python 3.12 via deadsnakes if not on 24.04+
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt install -y python3.12 python3.12-venv python3.12-dev

# Node.js via NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Kubernetes tooling
sudo snap install kubectl --classic
sudo snap install helm --classic
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube

# pre-commit
pip install --user pre-commit
```

Clone and bootstrap:

```bash
git clone https://github.com/stoopidco/stoopid-commons.git
cd stoopid-commons
make setup
```

### Windows

Native Windows is not supported. Use WSL2.

### WSL2 on Windows

Install WSL2 with Ubuntu from PowerShell as Administrator:

```powershell
wsl --install -d Ubuntu-22.04
```

Reboot when prompted. Launch Ubuntu from the Start menu and complete the initial user setup. Then follow the **Linux (Ubuntu/Debian)** instructions above inside the WSL2 environment.

For Docker Desktop integration with WSL2, install Docker Desktop on Windows and enable WSL2 integration under Settings → Resources → WSL Integration.

Clone the repo into your WSL2 home directory (not `/mnt/c/`) for filesystem performance:

```bash
cd ~
git clone https://github.com/stoopidco/stoopid-commons.git
cd stoopid-commons
make setup
```

## Usage

### Consuming packages in your project

`stoopid-commons` packages are published to public registries. Add them as normal dependencies — you do not need to clone this repo.

**Python (logging):**

```bash
pip install stoopid-logging
```

```python
from stoopid_logging import get_logger

log = get_logger(service="my-service")
log.info("processing started", record_id=123)
```

**TypeScript / Node (logging):**

```bash
npm install @stoopid/logging
```

```typescript
import { getLogger } from "@stoopid/logging";

const log = getLogger({ service: "my-service" });
log.info({ recordId: 123 }, "processing started");
```

**Rust (logging):**

```toml
[dependencies]
stoopid-logging = "0.1"
```

```rust
use stoopid_logging::init_logger;

init_logger("my-service");
tracing::info!(record_id = 123, "processing started");
```

**Helm chart base:**

```bash
helm repo add stoopid https://stoopidco.github.io/stoopid-commons-charts
helm repo update
```

In your service's `Chart.yaml`:

```yaml
dependencies:
  - name: stoopid-service-base
    version: "0.1.x"
    repository: "https://stoopidco.github.io/stoopid-commons-charts"
```

In your service's `values.yaml`, override only what differs from the base.

See the `examples/` directory in this repo for end-to-end usage examples per package.

### Local development workflows

If you're contributing to `stoopid-commons` itself:

```bash
make build      # Build all packages
make test       # Run all tests
make lint       # Lint all code
make fmt        # Format all code
make help       # Show all available targets
```

To test changes locally before publishing:

```bash
make publish-local PACKAGE=stoopid-logging-python
```

This publishes to a local registry that other repos on your machine can consume. See `CONTRIBUTING.md` for local registry setup details.

## Troubleshooting

### `make setup` fails on dependency installation

Confirm your toolchain versions match the requirements:

- Python ≥ 3.12
- Node.js ≥ 20
- Rust ≥ 1.75 (stable)

Run `make doctor` to check versions and report missing tools.

### Package not found when consuming

If `pip install stoopid-logging` or `npm install @stoopid/logging` fails:

- Verify the package name spelling
- Check for network connectivity to the registry
- For corporate networks, verify proxy/VPN configuration allows access to PyPI/npm
- Confirm the version constraint in your dependency file matches a published version

### Pre-commit hooks failing

```bash
pre-commit clean
pre-commit install --install-hooks
```

If a specific hook is failing without obvious cause, run it directly: `pre-commit run <hook-id> --all-files`.

### Helm chart base not pulling

```bash
helm repo update
helm dependency update  # in your service directory
```

If still failing, check the chart repository URL is correct and accessible.

### Cannot publish package locally

Verify your local registry is running. See `CONTRIBUTING.md` under "Local Package Registry" for Verdaccio (npm) and devpi (Python) setup.

### Filing an issue or bug report

Issues are welcome. File at https://github.com/stoopidco/stoopid-commons/issues with the `bug` label. Include:

- Your operating system and version
- Output of `make doctor` or relevant tool versions (`python --version`, `node --version`, `rustc --version`)
- The exact command that failed
- Full output of the failure
- Whether the issue reproduces in a clean clone of the repo

For security issues, see `SECURITY.md` — do not file public issues for security disclosures.

For feature requests or discussion, use the `enhancement` or `discussion` label.

## License

MIT. See `LICENSE`.

## Contributing

External contributions welcome. See `CONTRIBUTING.md` for development guidelines, code style, and the PR process. See `CODE_OF_CONDUCT.md` for community standards.
