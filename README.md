# Eligibility Engine MCP Server

> **Example Model Context Protocol (MCP) Server demonstrating leave assistance evaluation based on fictional regulations**

[![CI Pipeline](https://github.com/alpha-hack-program/eligibility-engine-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/alpha-hack-program/eligibility-engine-mcp-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

An example Model Context Protocol (MCP) server developed in Rust that demonstrates how to evaluate complex business rules using the ZEN Engine decision engine. This project serves as a reference implementation for building MCP servers with rule-based decision systems.

## ⚠️ **DISCLAIMER**

This example is based on fictional regulations from the imaginary Republic of Lysmark. For full details on the fictional legal framework, benefit scenarios, and eligibility rules used in this demonstration, please refer to the documents provided in the [`documents/`](./documents/) folder.

**This is a demonstration/example project only.** The regulations, amounts, and evaluation logic implemented here are fictional and created solely for educational and demonstration purposes. This software:

- **Should NOT be used for actual legal or administrative decisions**
- **Does NOT represent real government regulations**
- **Is NOT affiliated with any official government entity**
- **Serves as a technical example of MCP server implementation**

For real legal advice or official information about leave assistance, please consult official government sources and qualified legal professionals.

## 🎯 Features

- **5 Example Evaluation Scenarios**: Demonstrates implementation of complex rule sets (A-E)
- **Decision Engine Integration**: Shows how to use ZEN Engine for rule-based evaluation
- **Multiple Transport Protocols**: Examples of STDIO, SSE, and HTTP streamable implementations
- **Robust Input Validation**: Demonstrates JSON schema validation with detailed error handling
- **Production-Ready Containerization**: Example Docker/Podman setup for deployment
- **Claude Desktop Integration**: Example MCPB packaging for MCP integration
- **Professional Version Management**: Automated version sync with cargo-release
- **CI/CD Pipeline**: Comprehensive GitHub Actions workflow
- **Professional Repository Structure**: Organized scripts and clean project layout

## 📚 Quick Reference

| Task | Command | Description |
|------|---------|-------------|
| **🏗️ Build** | `make build-all` | Build all servers |
| **🧪 Test** | `make test` | Run all tests |
| **🚀 Release** | `make release-patch` | Create new patch release |
| **📦 Package** | `make pack` | Create Claude Desktop package |
| **🐳 Container** | `scripts/image.sh build` | Build container image |
| **🔄 Sync** | `make sync-version` | Sync versions manually |
| **ℹ️ Help** | `make help` | Show all commands |

## 📋 Example Assistance Scenarios (Fictional)

| Scenario | Description | Example Monthly Amount |
|----------|-------------|------------------------|
| **A** | Care for first-degree relative (illness/accident) | 725€ |
| **B** | Third child or more with newborn | 500€ |
| **C** | Adoption or foster care | 500€ |
| **D** | Multiple birth, adoption, or foster care | 500€ |
| **E** | Single-parent families | 500€ |

> **Note**: These scenarios and amounts are completely fictional and used only for demonstration purposes.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Cargo (included with Rust)
- `jq` for JSON processing ([Install jq](https://jqlang.github.io/jq/download/))
- `cargo-release` for version management: `cargo install cargo-release`

### Installation

```bash
# Clone the repository
git clone https://github.com/alpha-hack-program/eligibility-engine-mcp-rs.git
cd eligibility-engine-mcp-rs

# Build all servers
make build-all

# Or build individually
make build-sse      # SSE Server
make build-mcp      # MCP HTTP Server
make build-stdio    # STDIO Server for Claude
```

### Running

```bash
# SSE Server (recommended for development)
make test-sse

# MCP HTTP Server
make test-mcp

# Or directly
RUST_LOG=debug ./target/release/sse_server
```

## 🔧 Configuration

### Environment Variables

```bash
# Server configuration
RUST_LOG=info           # Logging level (debug, info, warn, error)

# Or use BIND_ADDRESS directly
BIND_ADDRESS=127.0.0.1:8000
```

### Example Usage

This input represents a request to the eligibility engine to determine if a person is eligible for unpaid leave benefits under Lysmark law, based on this query.

> "My wife just delivered our third baby and I'd like to know if I can request the unpaid leave aid."

```json
{
  `situation`: `birth`,
  `relationship`: `father`,
  `is_single_parent`: false,
  `total_children_after`: 3
}
```

**Applicant Profile:**
A father requesting leave for the birth of a child
Not a single parent (two-parent household)
Will have 2 children total after the birth

**Case Classification:**
Identified as "Case B: Third child or more with newborn"
Potentially eligible for a monthly benefit of 725€ if requirements are met


**Example Response:**
```json
{
  "output": {
    "description": "Third child or more with newborn",
    "monthly_benefit": 500,
    "additional_requirements": "The number of children must be 3 or more, the ages of at least 2 of the minors must be less than 6, if there is disability greater than 33% then the limit is 9 years",
    "case": "B",
    "potentially_eligible": true,
    "errores": [],
    "warnings": []
  },
  "input": {
    "relationship": "father",
    "situation": "birth",
    "is_single_parent": false,
    "total_children_after": 3.0
  },
  "relationship_valid": true
}
```

> **Important**: This is example data for demonstration purposes only.

## 🐳 Containerization

### Build and Run

This requires `podman` or `docker`. Configuration is managed through `.env` file.

```bash
# Build container image
scripts/image.sh build

# Run locally
scripts/image.sh run

# Run from remote registry
scripts/image.sh push
scripts/image.sh run-remote

# Show container information
scripts/image.sh info
```

### Environment Variables for Containers

```bash
# Production configuration
podman run -p 8001:8001 \
  -e BIND_ADDRESS=0.0.0.0:8001 \
  -e RUST_LOG=info \
  quay.io/atarazana/eligibility-engine-mcp-server:latest
```

## 📦 Claude Desktop Integration

### Packaging

```bash
# Create MCPB package for Claude Desktop
$ make pack
cargo build --release --bin stdio_server
   Compiling eligibility-engine-mcp-server v1.0.8 (/Users/.../eligibility-engine-mcp-rs)
    Finished `release` profile [optimized] target(s) in 18.23s
Packing MCP server for Claude Desktop...
chmod +x ./target/release/stdio_server
zip -rX eligibility-engine-mcp-server.mcpb -j mcpb/manifest.json ./target/release/stdio_server
updating: manifest.json (deflated 49%)
updating: stdio_server (deflated 63%)
```

### Example Claude Configuration

Drag and drop the `MCPB` file into the `Settings->Extensions` dropping area.

> **Note**: This demonstrates MCP integration patterns and is not intended for production use with real data.




## 🧪 Testing

```bash
# Run all tests
make test
```



## 🛠️ Development

### Available Commands

#### 🏗️ Build Commands
```bash
make build-all              # Build all servers
make build-mcp              # Build MCP server (streamable-http)
make build-sse              # Build SSE server
make build-stdio            # Build stdio server
make pack                   # Pack MCP server for Claude Desktop
```

#### 🚀 Release Commands (cargo-release)
```bash
make release-patch          # Create patch release (1.0.6 → 1.0.7)
make release-minor          # Create minor release (1.0.6 → 1.1.0)
make release-major          # Create major release (1.0.6 → 2.0.0)
make release-dry-run        # Show what release-patch would do
make sync-version           # Manually sync version to all files
```

#### 🧪 Test Commands
```bash
make test                   # Run all tests
make test-sse               # Test SSE server locally
make test-mcp               # Test MCP server locally
```

#### 🔧 Development Commands
```bash
make clean                  # Clean build artifacts
make help                   # Show all available commands
```

### Project Structure

```
├── src/                                    # Source code
│   ├── common/
│   │   ├── eligibility_engine.rs         # MCP logic and decision engine
│   │   └── mod.rs
│   ├── sse_server.rs                      # SSE Server
│   ├── mcp_server.rs                      # MCP HTTP Server
│   └── stdio_server.rs                    # STDIO Server
├── scripts/                               # Utility scripts
│   ├── sync-manifest-version.sh           # Version sync for cargo-release
│   └── image.sh                          # Container management script
├── mcpb/
│   └── manifest.json                      # Claude Desktop manifest
├── .github/workflows/                     # CI/CD pipelines
│   └── ci.yml                            # GitHub Actions workflow
├── docs/                                  # Documentation
├── .env                                   # Environment variables
├── Containerfile                          # Container definition
├── Cargo.toml                            # Rust package manifest
└── Makefile                              # Build commands
```

### Debug and Monitoring

First run the SSE server (or the Streamable HTTP version with `make test-mcp`):

```bash
$ make test-sse
cargo build --release --bin sse_server
   Compiling eligibility-engine-mcp-server v1.0.6 (/Users/cvicensa/Projects/rust/claude/eligibility-engine-mcp-rs)
    Finished `release` profile [optimized] target(s) in 18.26s
🧪 Testing SSE server...

RUST_LOG=debug ./target/release/sse_server
2025-09-22T16:53:01.931985Z  INFO sse_server: Starting sse Eligibility Engine MCP server on 127.0.0.1:8000
```

Second, run MCP inspector:

> **NOTE:** NodeJS 19+ has to be installed

```bash
$ make inspector
npx @modelcontextprotocol/inspector
Starting MCP inspector...
⚙️ Proxy server listening on 127.0.0.1:6277
🔑 Session token: 6f0fdc22e2a9775a95d60c976b37b873bffec1816002fc702ca8ec7186a7c338
Use this token to authenticate requests or set DANGEROUSLY_OMIT_AUTH=true to disable auth

🔗 Open inspector with token pre-filled:
   http://localhost:6274/?MCP_PROXY_AUTH_TOKEN=6f0fdc22e2a9775a95d60c976b37b873bffec1816002fc702ca8ec7186a7c338

🔍 MCP Inspector is up and running at http://127.0.0.1:6274 🚀
```

Open a browser and point to the URL with the token included.

Troubleshooting:

MCP error -32602: failed to deserialize parameters: missing field `is_single_parent`

Just click on the checkbox `is_single_parent` and try again.

Additional targets:

```bash
# Debug proxy
make proxy                  # Start mitmproxy on port 8888

# Supergateway for SSE
make sgw-sse               # STDIO -> SSE wrapping

# Supergateway for MCP
make sgw-mcp               # STDIO -> MCP HTTP wrapping
```

## 📚 API Reference

### Main Endpoint

**POST** `/message` - Example endpoint for rule evaluation

### Example Input Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `relationship` | string | ✅ | Family relationship (father, mother, son, daughter, spouse, partner, husband, wife, woman, man) |
| `situation` | string | ✅ | Care reason (birth, adoption, foster_care, illness, accident, etc.) |
| `is_single_parent` | boolean | ✅ | Is it a single-parent family? |
| `total_children_after` | number | ❌ | Number of children (optional, required for Case B) |

### Response

| Field | Type | Description |
|-------|------|-------------|
| `case` | string | Applicable scenario letter (A-E) |
| `description` | string | Scenario description |
| `monthly_benefit` | number | Monthly amount in euros |
| `potentially_eligible` | boolean | Meets basic requirements? |
| `additional_requirements` | string | Additional specific requirements |
| `errores` | array | List of validation errors |
| `warnings` | array | Warnings and additional information |

## 🔒 Security

- **Input validation**: Strict JSON schemas
- **Non-root user**: Containers run as user `1001`
- **Security audit**: `cargo audit` in CI/CD
- **Minimal image**: Based on UBI 9 minimal

## 🤝 Contributing

### Development Workflow

1. **Fork the project**
2. **Create feature branch**: `git checkout -b feature/new-feature`
3. **Make changes and test**: `make test`
4. **Commit changes**: `git commit -am 'Add new feature'`
5. **Push to branch**: `git push origin feature/new-feature`
6. **Create Pull Request**

### Professional Release Process

1. **Development**: Make changes, test with `make test`
2. **Version Bump**: Use `make release-patch/minor/major`
3. **Build**: Use `make pack` for Claude Desktop integration
4. **Container**: Use `scripts/image.sh build` for containerization

### Guidelines

- **Code Quality**: Follow `cargo fmt` and pass `cargo clippy`
- **Testing**: Add tests for new functionality
- **Version Management**: Let cargo-release handle versioning
- **CI/CD**: Ensure all GitHub Actions pass
- **Documentation**: Update README.md as needed
- **Professional Structure**: Keep scripts in `scripts/` directory

## ⚙️ Version Management

This project uses **cargo-release** for professional version management with automatic synchronization across all configuration files.

### 🔄 Version Sync System

- **Single Source of Truth**: `Cargo.toml` version controls everything
- **Automatic Sync**: Updates `mcpb/manifest.json` and `.env` automatically
- **Git Integration**: Creates commits and tags automatically

### 📦 Release Workflow

```bash
# 1. Make your changes and commit them
git add -A && git commit -m "feat: your changes"

# 2. Create a release (choose appropriate version bump)
make release-patch     # Bug fixes: 1.0.6 → 1.0.7
make release-minor     # New features: 1.0.6 → 1.1.0  
make release-major     # Breaking changes: 1.0.6 → 2.0.0

# 3. Build and package
make pack
scripts/image.sh build
scripts/image.sh push

# 4. Push to repository
git push && git push --tags
```

### 🔍 Preview Changes

```bash
# See what would happen without making changes
make release-dry-run
```

### 🛠️ Manual Version Sync (Development)

```bash
# Sync version from Cargo.toml to other files manually
make sync-version
```

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## 🚀 Production Deployment

### Environment Configuration

The project uses `.env` for environment management:

```bash
# Version (automatically managed by cargo-release)
VERSION=1.0.6

# Container Configuration  
APP_NAME="eligibility-engine-mcp-rs"
BASE_IMAGE="registry.access.redhat.com/ubi9/ubi-minimal"
PORT=8001

# Registry Configuration
REGISTRY=quay.io/atarazana
```

### CI/CD Pipeline

The project includes a comprehensive GitHub Actions workflow:
- ✅ **Automated Testing**: Unit tests and integration tests
- ✅ **Version Sync Validation**: Tests cargo-release functionality  
- ✅ **Container Building**: Tests containerization process
- ✅ **Artifact Management**: Builds and uploads release artifacts
- ✅ **Cross-platform Support**: Tests on Ubuntu with multiple container runtimes

## 🙋 Support

- **Issues**: [GitHub Issues](https://github.com/alpha-hack-program/eligibility-engine-mcp-rs/issues)
- **Documentation**: [Project Wiki](https://github.com/alpha-hack-program/eligibility-engine-mcp-rs/wiki)
- **CI/CD**: Automated testing and deployment via GitHub Actions

## 🏷️ Tags

`mcp` `model-context-protocol` `rust` `eligibility-engine` `unpaid-leave` `zen-engine` `claude` `decision-engine` `cargo-release` `professional-rust` `containerization` `ci-cd`

---

**Developed with ❤️ by [Alpha Hack Group](https://github.com/alpha-hack-program)**