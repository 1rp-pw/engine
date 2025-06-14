# Policy Engine Testcontainer - Go Example

This example shows how to use the Policy Engine as a testcontainer in Go applications, following the same pattern as other testcontainers like PostgreSQL.

## Setup

1. Make sure you have Docker running
2. Build the policy engine image:
   ```bash
   cd ../../..  # Go back to policy engine root
   docker build -t policy-engine:latest .
   ```

## Running Tests

```bash
go mod tidy
go test -v
```

## Usage Pattern

The Go testcontainer follows the exact same pattern as PostgreSQL testcontainers:

```go
func setupPolicyEngine(ctx context.Context) (*PolicyEngineContainer, error) {
    req := testcontainers.ContainerRequest{
        Image:        "policy-engine:latest",
        ExposedPorts: []string{"3000/tcp"},
        Env: map[string]string{
            "FF_ENV_ID":     "test-env",
            "FF_AGENT_ID":   "test-agent",
            "FF_PROJECT_ID": "test-project",
        },
        WaitingFor: wait.ForHTTP("/health").
            WithPort("3000/tcp").
            WithStartupTimeout(60 * time.Second),
    }

    container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
        ContainerRequest: req,
        Started:          true,
    })
    // ... handle container setup
}

func TestPolicyEvaluation(t *testing.T) {
    ctx := context.Background()

    // Start Policy Engine container
    pe, err := setupPolicyEngine(ctx)
    assert.NoError(t, err)
    defer func() {
        if pe != nil {
            if err := pe.Terminate(ctx); err != nil {
                t.Logf("failed to terminate container: %v", err)
            }
        }
    }()
    assert.NotNil(t, pe)

    // Use the policy engine for testing
    response, err := pe.EvaluatePolicy(ctx, 
        "A **User** gets access if the __role__ of the **User** is equal to \"admin\".",
        map[string]interface{}{"role": "admin"},
        false)
    assert.NoError(t, err)
    // ... test your policy logic
}
```

## Features

- **Same Pattern as PostgreSQL**: Uses identical testcontainer setup pattern
- **Automatic Cleanup**: Container is automatically terminated after tests
- **Health Checks**: Built-in health checking before tests run
- **Easy Policy Testing**: Simple API for policy evaluation
- **Concurrent Safe**: Each test gets its own container instance
- **Configurable**: Environment variables for feature flags
- **Timeout Handling**: Proper startup timeout configuration

## API Methods

### `setupPolicyEngine(ctx context.Context) (*PolicyEngineContainer, error)`
Creates and starts a new Policy Engine testcontainer, similar to your PostgreSQL setup.

### `EvaluatePolicy(ctx context.Context, rule string, data interface{}, trace bool) (*PolicyResponse, error)`
Evaluates a policy rule against data, with optional tracing.

### `HealthCheck(ctx context.Context) error`
Verifies the container is ready to accept requests.

## Test Examples

The example includes several test patterns:
- Basic connectivity testing
- Single policy evaluation
- Complex policies with nested data
- Multiple policy testing in sequence
- Error handling and edge cases