
## Smoketest Harness
Smoke tests simulate a network by starting a "swarm" of validators locally on the dev machine. The harnes:
- simulates a genesis (note this is not the 0L migration genesis, but one tailor made for testing)
-  exposes some apis that can manage the swarm (start stop individual nodes).
It provides introspection into the state of each node.
- It also provides bindings to the rust SDK such that transactions can be executed in the VM (by the root account or other accounts).
- It also provides introspection into the account state (of the root or any account).

## How to use this
The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.
Don't add functionality tests in this module. The tests here are meta tests to show the harness itself works. Let's keep the dependencies minimal here, to prevent cycling.