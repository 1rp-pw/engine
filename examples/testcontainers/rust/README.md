# Policy Engine Testcontainer - Rust Example

This example shows how to use the Policy Engine as a testcontainer in Rust applications.

## Setup

1. Make sure you have Docker running
2. Build the policy engine image:
   ```bash
   cd ../../..  # Go back to policy engine root
   docker build -t policy-engine:latest .
   ```

## Running Tests

```bash
cargo test
```

## Usage in Your Tests

```rust
use policy_engine_testcontainer::PolicyEngineContainer;
use testcontainers::clients::Cli;
use serde_json::json;

#[tokio::test]
async fn my_integration_test() {
    let docker = Cli::default();
    let policy_engine = PolicyEngineContainer::new(&docker);
    
    let result = policy_engine.evaluate_policy(
        "A **User** gets access if the __role__ of the **User** is equal to \"admin\".",
        json!({"role": "admin"}),
        false
    ).await.unwrap();
    
    // Assert your expectations
    assert!(result.to_string().contains("access"));
}
```

## Features

- Automatic container lifecycle management
- Built-in policy evaluation helper methods
- Configurable feature flag environment variables
- Support for both traced and non-traced evaluations
- Clean async/await interface