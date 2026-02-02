.PHONY: all clean build-mcp build-http pack-mcp pack-http test-http release-patch release-minor release-major release-dry-run

all: build-all

# Build MCP server (streamable-http)
build-mcp:
	cargo build --release --bin mcp_server

# Build stdio server (stdio)
build-stdio:
	cargo build --release --bin stdio_server

# Build all servers
build-all: build-mcp build-stdio

# Pack MCP server for Claude Desktop
pack: build-stdio
	@echo "Packing MCP server for Claude Desktop..."
	chmod +x ./target/release/stdio_server
	zip -rX eligibility-engine-mcp-server.mcpb -j mcpb/manifest.json ./target/release/stdio_server

# Test MCP server locally
test-mcp: build-mcp
	@echo "🧪 Testing MCP server..."
	@echo ""
	RUST_LOG=debug BIND_ADDRESS=0.0.0.0:8001 ./target/release/mcp_server
	
clean:
	rm -f *.mcpb *.zip
	cargo clean

inspector:
	npx @modelcontextprotocol/inspector

test:
	@echo "Running all tests..."
	cargo test --bin stdio_server

# Release management with cargo-release
release-patch: 
	@echo "🚀 Creating patch release (x.y.Z+1)..."
	cargo release patch --execute

release-minor: 
	@echo "🚀 Creating minor release (x.Y+1.0)..."
	cargo release minor --execute

release-major: 
	@echo "🚀 Creating major release (X+1.0.0)..."
	cargo release major --execute

release-dry-run: 
	@echo "🔍 Dry run - showing what would happen..."
	cargo release patch --dry-run

# Manual version sync (for development)
sync-version:
	@echo "🔄 Manually syncing version..."
	scripts/sync-manifest-version.sh

help:
	@echo "Usage:"
	@echo ""
	@echo "🏗️  Build Commands:"
	@echo "  make all           - Build all servers"
	@echo "  make build-mcp     - Build MCP server (streamable-http)"
	@echo "  make build-stdio   - Build stdio server" 
	@echo "  make build-all     - Build all servers"
	@echo "  make pack          - Pack MCP server for Claude Desktop"
	@echo ""
	@echo "🚀 Release Commands (uses cargo-release):"
	@echo "  make release-patch - Create patch release (1.0.6 → 1.0.7)"
	@echo "  make release-minor - Create minor release (1.0.6 → 1.1.0)"
	@echo "  make release-major - Create major release (1.0.6 → 2.0.0)"
	@echo "  make release-dry-run - Show what release-patch would do"
	@echo "  make sync-version  - Manually sync version to manifest.json"
	@echo ""
	@echo "🧪 Test Commands:"
	@echo "  make test-mcp      - Test MCP server locally"
	@echo "  make test          - Run all tests"
	@echo ""
	@echo "🔧 Utility Commands:"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make inspector     - Start Model Context Protocol Inspector"
	@echo "  make help          - Show this help message"