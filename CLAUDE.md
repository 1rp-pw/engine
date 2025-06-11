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

## Architecture Overview

This is a Policy Engine that evaluates business rules written in a custom DSL against JSON data. The system uses a Pest parser to convert human-readable rules into executable conditions.

### Core Flow
1. **Parser** (`runner/parser/`) - Converts DSL text to structured rules using Pest grammar
2. **Evaluator** (`runner/evaluator/`) - Executes rules against JSON data with tracing support
3. **HTTP API** (`main.rs`) - Axum server exposing POST endpoint for rule evaluation

### Key Design Patterns
- **Grammar Composition**: Multiple `.pest` files are combined at build time via `build.rs`, never alter grammar.pest directly
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