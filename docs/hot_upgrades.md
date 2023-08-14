
# NETWORK HOT UPGRADES
# Overview
The "framework" which contains all the consensus, account, econ policies, etc. for the network is written in Move. This code is stored in the network database, and effectively executed on demand. This means that framework upgrades can occur without redeploying the Move VM itself, or the supporting system code (network node software). It also means the state machine can upgrade without a coordinated halt.
To do this we require two libra tools: `framework` for building the artifacts, and `txs` for proposing, voting, and ultimately deploying the artifacts.

# Procedure
## Build Artifacts

### 1. Build a new head.mrb bundle for the framework into `./framework/releases`.
```
cargo r --release release
```

### 2. Build the upgrade Move transaction scripts

This will be a Move package which is machine-generated for a one-time execution. It contains bytecode which will be allowed to be executed (by anyone), once there is a vote and agreement on the proposal passing. The on-chain execution is guarded a hash of this transction, which the proposer provides in the proposal transaction (in advance of the vote).

An upgrade script that is tampered with will yield a different execution hash, and will be prevented from running (it is likely to be blocked by the transaction size limits before entering the mempool).

The `framework upgrade` command will produce a newly compiled Move upgrade transaction script, its binary, and the hash.

You need to provide:
- `--output-dir`: this directory the upgrade transaction files should be saved to. A new folder called `framework_upgrade` will be created under the output-dir path.

- `--framework-local-dir`: the source code for the framework so that the transaction script can import it as a dependency.

- `--mrb-path`: the compiled bundle of the framework from step #1. By default this is under ``./framework/releases/head.mrb``

```
# Note the paths
cargo r --release upgrade \
--output-dir ./framework/releases \
--framework-local-dir ./framework/libra-framework \
--mrb-path ./framework/releases/head.mrb
```

All the artifacts are now created, the proposal transaction can be submitted. But it's a good idea to document this on github first.

### 3. Share the output artifacts on Github.

Create a new repository with the outputted directory. Add a README.md file.

The proposer can add the link to this Github repo in the proposal phase.


## Upgrade Ceremony
### 4. With `txs` anyone (no authority needed) can submit the proposal and metadata. You'll need to provide the actual script compiled path, and an optional URL which contains documentation of the proposal (e.g github).

```
# note the actual script dir
txs upgrade propose --script-dir ./framework/release/framework_upgrade --metadata-url <URL>
```
If this transaction succeeds it will produce a proposal id, which is a number. Now the proposal is open to voting.

### 5. With `txs` anyone with governance authority (the epoch's validators as of `V7`), can submit a vote in favor (or against it with `--should-fail`).

We assume the default is to vote in favor. To vote "approve" simply:
```
txs vote --proposal-id <number>
```

If voter would like the proposal to be rejected:
```
txs vote --proposal-id <number> --should-fail
```

After everyone has voted (to reach the consensus threshold of 66% as of  `V7`), the proposal will be in a "Resolvable" state. Anyone can resolve it by submitting the upgrade transaction. This means the sender must have the source transaction script for the upgrade (step #2 above).

### 6. Use `txs` to resolve a successfully approved proposal
```
# Note the actual path
txs resolve --proposal-script-dir ./framework/release/framework_upgrade
```

If this transction is successful the new bytecode will be written to the VM, and a new epoch will be triggered.
