# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
```bash
cargo build
cargo build --release
```

### Run Tests
```bash
cargo test                    # Run all tests
cargo test test_name         # Run specific test
cargo test -- --nocapture    # Show println! output
```

### Lint and Format
```bash
cargo fmt                    # Format code
cargo fmt -- --check        # Check formatting without changes
cargo clippy                # Run linter
cargo clippy -- -D warnings # Treat warnings as errors
```

### Run Server
```bash
cargo run                   # Run with default port 3000
PORT=8080 cargo run        # Run with custom port
```

### Docker Commands
```bash
# Build Docker image
docker build -t policy-engine:latest .

# Run container
docker run -p 3000:3000 \
  -e FF_ENV_ID="test-env" \
  -e FF_AGENT_ID="test-agent" \
  -e FF_PROJECT_ID="test-project" \
  policy-engine:latest

# Run with docker-compose
docker-compose up           # Start the service
docker-compose up -d        # Start in background
docker-compose down         # Stop the service

# Run test profile
docker-compose --profile test up
```

## Architecture Overview

This is a Policy Engine that evaluates business rules written in a custom DSL against JSON data. The system uses a Pest parser to convert human-readable rules into executable conditions.

### Core Flow
1. **Parser** (`runner/parser/`) - Converts DSL text to structured rules using Pest grammar
2. **Evaluator** (`runner/evaluator/`) - Executes rules against JSON data with tracing support
3. **HTTP API** (`main.rs`) - Axum server exposing POST endpoint for rule evaluation

### Key Design Patterns
- **Grammar Composition**: Multiple `.pest` files are combined at build time via `build.rs`, always ignore grammar.pest since it's compiled at build
- **Error Tracing**: All evaluations can produce detailed execution traces for debugging
- **Property Transformation**: Automatic conversion between snake_case and camelCase for JSON access
- **Caching**: Performance optimizations through selector mapping in RuleSet
- **Golden Rule**: There is only one golden rule, you can have as many references to sub policies, but only one overruling policy that all must eventually reduce down to

### DSL Syntax Examples
```
A **Person** gets senior_discount
  if the __age__ of the **Person** is greater than or equal to 65.

A **Order** gets expedited_shipping
  if the __total__ of the **Order** is greater than 100
  and the __membership_level__ of the **Customer** is in ["gold", "platinum"].
```

### Testing Approach
Tests are embedded in `src/lib.rs` covering all operators, property access patterns, and edge cases. When adding new operators or functionality, follow the existing test pattern with both positive and negative test cases.

**Important**: Always add tests to the relevant `lib.rs` file in the module where the functionality is implemented, rather than creating separate test files. This keeps tests close to the code they're testing and maintains consistency with the existing test structure.

## Testcontainer Integration

The Policy Engine can be used as a testcontainer for integration testing in other systems.

### Rust Testcontainer Example
```rust
use policy_engine_testcontainer::PolicyEngineContainer;
use testcontainers::clients::Cli;
use serde_json::json;

#[tokio::test]
async fn test_policy_integration() {
    let docker = Cli::default();
    let policy_engine = PolicyEngineContainer::new(&docker);
    
    let result = policy_engine.evaluate_policy(
        "A **User** gets access if the __role__ of the **User** is equal to \"admin\".",
        json!({"role": "admin"}),
        false
    ).await.unwrap();
    
    assert!(result.to_string().contains("access"));
}
```

### Setup for Integration Testing
1. Build the Docker image: `docker build -t policy-engine:latest .`
2. Use the testcontainer in your integration tests
3. The container automatically configures feature flag environment variables
4. Access the policy evaluation endpoint at the container's mapped port

See `examples/testcontainers/rust/` for complete implementation examples.