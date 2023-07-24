
## Smoketest Harness
Smoke tests simulate a network by starting a "swarm" of validators locally on the dev machine. The harnes:
- simulates a genesis (note this is not the 0L migration genesis, but one tailor made for testing)
-  exposes some apis that can manage the swarm (start stop individual nodes).
It provides introspection into the state of each node.
- It also provides bindings to the rust SDK such that transactions can be executed in the VM (by the root account or other accounts).
- It also provides introspection into the account state (of the root or any account).

## How to write tests
The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.
Don't add functionality tests in this module. The tests here are meta tests to show the harness itself works. Let's keep the dependencies minimal here, to prevent cycling.

## How to run tests

### Node Binary
Any tests that depend on `libra-smoke-tests` will require that there is a compiled version of `zapatos-node` locally.

For the tests to find the binary's path an environment variable must be set `ZAPATOS_BIN_PATH`

So either export that variable in the test environment (CI), or run inline with `ZAPATOS_BIN_PATH=path/to/target/release/zapatos_node`

### Stack Overflow

Since the smoke tests can have nested operations, it's possible that a test function could exceed its stack (e.g. `txs smoke_publish()`). At test runtime you can change the stack size with a rust env variable `RUST_MIN_STACK`

Run with:
```
RUST_MIN_STACK=104857600 cargo t smoke_publish
```

### expected error
If you did not export the variable you will see:
```
'failed to read env.var. ZAPATOS_BIN_PATH: NotPresent',
```