#!/bin/bash

# Pre-release hook for cargo-release
# Syncs the version from Cargo.toml to dxt/manifest.json and .env
# Also updates Rust source code with .env values

set -e  # Exit on any error

echo "🔄 Syncing version from Cargo.toml to dxt/manifest.json, .env, and Rust source..."

# Get version from Cargo.toml using cargo metadata
if command -v jq &> /dev/null; then
    VERSION=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | jq -r '.packages[0].version')
else
    # Fallback to grep if jq is not available
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
fi

if [ -z "$VERSION" ] || [ "$VERSION" == "null" ]; then
    echo "❌ Error: Could not extract version from Cargo.toml!"
    exit 1
fi

echo "📦 Found version: $VERSION"

# Update dxt/manifest.json
if [ -f "dxt/manifest.json" ]; then
    if command -v jq &> /dev/null; then
        # Use jq for robust JSON editing
        jq --arg version "$VERSION" '.version = $version' dxt/manifest.json > dxt/manifest.json.tmp && mv dxt/manifest.json.tmp dxt/manifest.json
        echo "✅ Updated dxt/manifest.json version to $VERSION"
    else
        echo "⚠️  jq not found. Please install jq for robust JSON editing:"
        echo "   macOS: brew install jq"
        echo "   Ubuntu: sudo apt install jq"
        exit 1
    fi
else
    echo "⚠️  dxt/manifest.json not found - skipping"
fi

# Update .env file with VERSION
if [ -f ".env" ]; then
    # Update existing .env file
    if grep -q "^VERSION=" .env; then
        # Replace existing VERSION line
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed syntax
            sed -i '' "s/^VERSION=.*/VERSION=$VERSION/" .env
        else
            # Linux sed syntax
            sed -i "s/^VERSION=.*/VERSION=$VERSION/" .env
        fi
        echo "✅ Updated .env VERSION to $VERSION"
    else
        # Add VERSION to existing .env
        echo "VERSION=$VERSION" >> .env
        echo "✅ Added VERSION=$VERSION to .env"
    fi
else
    # Create new .env file with defaults
    cat > .env << EOF
APP_NAME=eligibility-engine-mcp-rs
VERSION=$VERSION
TITLE=Eligibility Engine MCP Server
EOF
    echo "✅ Created .env with default values"
fi

# Read values from .env file for Rust source replacement
echo "🔧 Reading .env values for Rust source code replacement..."

# Function to read .env variables
read_env_var() {
    local var_name=$1
    local default_value=$2
    local value
    
    if [ -f ".env" ]; then
        value=$(grep "^${var_name}=" .env | cut -d'=' -f2- | sed 's/^"//' | sed 's/"$//')
    fi
    
    if [ -z "$value" ]; then
        value=$default_value
    fi
    
    echo "$value"
}

# Get values from .env
APP_NAME=$(read_env_var "APP_NAME" "eligibility-engine-mcp-rs")
ENV_VERSION=$(read_env_var "VERSION" "$VERSION")  # Use VERSION from Cargo.toml as fallback
TITLE=$(read_env_var "TITLE" "Eligibility Engine")

echo "📋 .env values:"
echo "   APP_NAME: $APP_NAME"
echo "   VERSION: $ENV_VERSION" 
echo "   TITLE: $TITLE"

# Update Rust source code
RUST_FILE="src/common/eligibility_engine.rs"
if [ -f "$RUST_FILE" ]; then
    echo "🦀 Updating Rust source code with .env values..."
    
    # Create a backup
    cp "$RUST_FILE" "$RUST_FILE.bak"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS sed syntax
        sed -i '' "s/std::env::var(\"APP_NAME\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$APP_NAME\".to_string()/g" "$RUST_FILE"
        sed -i '' "s/std::env::var(\"VERSION\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$ENV_VERSION\".to_string()/g" "$RUST_FILE"
        sed -i '' "s/std::env::var(\"TITLE\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$TITLE\".to_string()/g" "$RUST_FILE"
    else
        # Linux sed syntax
        sed -i "s/std::env::var(\"APP_NAME\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$APP_NAME\".to_string()/g" "$RUST_FILE"
        sed -i "s/std::env::var(\"VERSION\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$ENV_VERSION\".to_string()/g" "$RUST_FILE"
        sed -i "s/std::env::var(\"TITLE\")\.unwrap_or_else(|_| \"[^\"]*\"\.to_string())/\"$TITLE\".to_string()/g" "$RUST_FILE"
    fi
    
    echo "✅ Updated $RUST_FILE with .env values"
    echo "💾 Backup saved as $RUST_FILE.bak"
else
    echo "⚠️  $RUST_FILE not found - skipping Rust source update"
fi

echo "🎉 Version sync complete!"
