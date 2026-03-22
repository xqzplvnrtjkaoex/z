# Start dev databases
db-up:
    docker compose up -d

# Stop dev databases
db-down:
    docker compose down

# Stop dev databases and remove volumes
db-reset:
    docker compose down -v

# Run all tests
test:
    cargo test --workspace

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Lint
lint:
    cargo clippy --workspace

# Format check
fmt-check:
    cargo fmt --check

# Build all
build:
    cargo build --workspace
