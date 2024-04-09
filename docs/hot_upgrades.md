
# Network Hot Upgrades

## Overview

The "framework" which contains all the consensus, account, econ policies, etc. for the network is written in Move. This code is stored in the network database, and effectively executed on demand. This means that framework upgrades can occur without redeploying the Move VM itself, or the supporting system code (network node software). It also means the state machine can upgrade without a coordinated halt.

- To do this we require the `libra` cli tool. The subcommand `libra move framework` is used for building the artifacts, and `libra txs` for proposing, voting, and ultimately deploying the artifacts.

## TLDR
- **Fetch the latest release**: `cd libra-framework; git fetch --all; git checkout release-x.x.x`
- **Build framework**: `libra move framework  upgrade --output-dir ~/framework_upgrade --framework-local-dir ~/libra-framework/framework/`
- **Propose**: `libra txs governance propose --proposal-script-dir ~/framework_upgrade/1-move-stdlib/ --metadata-url https://www.github.com/0LNetworkCommunity/UpdateProposalTemplate`
- **Validators vote**: `libra txs governance  vote --proposal-id <ID>`
- **Resolve**:

    1. `libra txs governance resolve --proposal-script-dir ~/framework_upgrade/1-move-stdlib/ --proposal-id 0`

    2. `libra txs governance resolve --proposal-script-dir ~/framework_upgrade/2-vendor-stdlib/ --proposal-id 0`

    3. `libra txs governance resolve --proposal-script-dir ~/framework_upgrade/3-libra-framework/ --proposal-id 0`

## Upgrade Types: Single and Multiple

When performing upgrades within the network framework, two primary approaches can be employed: single and multiple module upgrades. The key distinction lies in how these upgrades are proposed, voted on, and executed, particularly with respect to handling multiple modules.

### Single Framework Upgrade
A single framework upgrade involves updating a singular set of modules within the framework. This process is straightforward and includes the following steps:

- **Build Framework**: Generate the upgrade Move transaction scripts for the module.
- **Proposal Submission**: Propose the upgrade for the specific framework module.
- **Validator Voting**: Validators within the network vote for or against the proposed upgrade.
- **Achieving Consensus**: The proposal moves forward once it receives support from at least 66% of active validators.
- **Resolution and Deployment**: Resolve the proposal using the exact framework directory that matches the build of the proposed upgrade. This can be done by any validator using the same release branch from the `libra-framework` repository

### Multiple Framework Upgrades
Multiple framework upgrades require a more nuanced approach, especially regarding resolution stages, to ensure a coherent and secure update across several modules.

- **Build Framework**: Similar to a single upgrade, start by generating Move transaction scripts for all relevant modules.
- **Proposal for Initial Module**: Propose the upgrade by using the first module (`1-move-stdlib`). This initial proposal is critical as it kickstarts the governance process for the entire upgrade.

  Importantly, the transaction script for upgrading this first module includes a significant addition: **the transaction hash for the subsequent modules** that needs upgrading. These hashes, produced during the artifact building phase, serve as secure identifiers for each module's upgrade script.

- **Validator Voting**: As with single upgrades, validators vote for or against the proposed upgrade.
- **Achieving Consensus and Sequential Resolution**: Once at least 66% of active validators support the proposal, the initial upgrade can be resolved.
- **Sequential Upgrade Execution**: Execute the resolution process for all involved modules, following the order 1-3.


## Upgrade Policy
The diem framework has information in the metadata that controls the policy that a publisher can set for upgrading their modules. These are:
- Arbitrary(0): Allows code upgrades without compatibility checks, for non-shared packages.
- Compatible(1): Requires compatibility checks to ensure no breaking changes in public functions or resource layouts.
- Immutable(2): Prevents any upgrades, ensuring the permanence of critical code.

Due to some circumstances, a publisher may want to downgrade the policy to allow changes to go through that a restrictive policy would not allow. We can do this by building the framework with a flag(`--danger-force-upgrade`) that sets the upgrade policy as Aribitrary

Example

```
libra txs governance propose --proposal-script-dir ~/framework_upgrade/3-libra-framework/ --metadata-url https://www.github.com/0LNetworkCommunity/UpdateProposalTemplate --danger-force-upgrade
```

## Procedure

We will use this guide to do a multi-step upgrade as this is the most common upgrade that is done.

### Build Artifacts

##### 1. Build the upgrade Move transaction scripts

This will be a Move package which is machine-generated for a one-time execution. It contains bytecode which will be allowed to be executed (by anyone), once there is a vote and agreement on the proposal passing. The on-chain execution is guarded with a hash of this transaction, which the proposer provides in the proposal transaction (in advance of the vote).

An upgrade script that is tampered with will yield a different execution hash, and will be prevented from running (it is likely to be blocked by the transaction size limits before entering the mempool).

The `libra move framework upgrade` command will produce a newly compiled Move upgrade transaction script, its binary, and the hash.

You need to provide:
- `--output-dir`: this directory the upgrade transaction files should be saved to. A new folder called `framework_upgrade` will be created under the output-dir path.

- `--framework-local-dir`: the source code for the framework so that the transaction script can import it as a dependency.

Optionally you could provide the flag `--danger-force-upgrade

```
# Note the paths
libra move framework upgrade --output-dir <OUTPUT_DIR> --framework-local-dir <FRAMEWORK_PATH>

# Example
libra move framework upgrade --output-dir ~/framework_upgrade --framework-local-dir ~/libra-framework/framework/
```
:::note
This creates 3 seperate library upgrade script directories
- 1-move-stdlib
- 2-vendor-stdlib
- 3-libra-framework

You will choose depending on which library you want updated
:::

All the artifacts are now created, the proposal transaction can be submitted. But it's a good idea to document this on github first.

##### 2. Share the output artifacts on Github.

Create a new repository with the outputted directory. Add a README.md file.

The proposer can add the link to this Github repo in the proposal phase.


### Upgrade Ceremony

##### 3. With `txs` anyone (no authority needed) can submit the proposal and metadata. You'll need to provide the actual script compiled path, and an optional URL which contains documentation of the proposal (e.g github).

```
# note the actual script dir
libra txs governance propose --proposal-script-dir <PROPOSAL_SCRIPT_DIR> --metadata-url <URL>

# Example
libra txs governance propose --proposal-script-dir ~/framework_upgrade/1-move-stdlib/ --metadata-url https://www.github.com/0LNetworkCommunity/UpdateProposalTemplate

```
If this transaction succeeds it will produce a proposal id, which is a number. Now the proposal is open to voting.

:::note
You can query the next proposal using this command: ` libra query view --function-id 0x1::diem_governance::get_next_governance_proposal_id`
:::

##### 4. With `libra txs` anyone with governance authority (the epoch's validators as of `V7`), can submit a vote in favor (or against it with `--should-fail`).

We assume the default is to vote in favor. To vote "approve" simply:
```
libra txs governance  vote --proposal-id <PROPOSAL_ID>
```

If voter would like the proposal to be rejected:
```
libra txs governance vote --proposal-id <PROPOSAL_ID> --should-fail
```
:::note
You can query to see the for and against votes using this command: ` libra query view --function-id 0x1::diem_governance::get_votes --args <proposal_number>`
:::

After everyone has voted (to reach the consensus threshold of 66% as of  `V7`), the proposal will be in a "Resolvable" state. Anyone can resolve it by submitting the upgrade transaction. This means the sender must have the source transaction script for the upgrade (step #1 above).

##### 6. Use `txs` to resolve a successfully approved proposal
```
# Note the actual path
libra txs governance resolve --proposal-script-dir <PROPOSAL_SCRIPT_DIR> --proposal-id <PROPOSAL_ID>

# Example
    1. libra txs governance resolve --proposal-script-dir ~/framework_upgrade/1-move-stdlib/ --proposal-id 0

    2. libra txs governance resolve --proposal-script-dir ~/framework_upgrade/2-vendor-stdlib/ --proposal-id 0

    3. libra txs governance resolve --proposal-script-dir ~/framework_upgrade/3-libra-framework/ --proposal-id 0
```

If this transaction is successful the new bytecode will be written to the VM
