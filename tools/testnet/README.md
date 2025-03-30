# Libra Framework Testnet Tools

This package provides tools for creating and managing test networks for the Libra Framework, allowing both clean-slate testing and production-identical testing environments.

# Quick Start
``` bash
# make sure you have `libra` compiled and exported
export DIEM_FORGE_NODE_BIN_PATH=$HOME/.cargo/bin/libra

# assuming you are using source you want to have the latest Move framework MRB ready
# the code defaults to <source code path>/framework/releases/head.mbr
cd libra-framework && libra move framework release

```

## Smoke - starts a managed local testnet
``` bash
# run locally a virgin testnet with alice, bob, carol default test accounts
libra testnet smoke

# start local twin of mainnet forking from an archive backup at epoch number
libra testnet --twin_epoch_restore=344 smoke

# export a json file so you know the paths and urls needed to play
libra testnet --json-file ./testnet.json smoke

# Using prebuilt Move framework assuming you have downloaded a file like `release-7.0.3.mrb`
curl -LO https://github.com/0LNetworkCommunity/libra-framework/releases/download/7.0.3/release-7.0.3.mrb

libra testnet --framework-mrb-path ./release-7.0.3.mrb smoke
```

## Config only - configure the data so you can start nodes yourself on independent hosts or containers.
``` bash
# configure a testnet with alice, bob, carol test accounts, and assign hosts to them in (ordered). Do this on each host.

libra testnet \
--test-dir ./saved_here/ \
configure --host 0.0.0.1:6180 --host 0.0.0.2:6180 --host 0.0.0.3:6180

# then on each host start the node as you would in production, with each host choosing one of the test personas by path.
libra node --config-path ./saved_here/alice/validator.yaml
```
## Core Features

The testnet tools support two key operational modes:

1. **Virgin Mode**: Create a fresh testnet with a new genesis and no prior state
2. **Twin Mode**: Fork an existing mainnet, creating a clone with identical state for realistic testing

These modes can be applied to either container-based or swarm-based testnets.

## Testing Approaches

### Container-based Testnet

The container-based approach uses Docker Compose to create a testnet that closely resembles a production environment:

- Uses actual production binaries
- Provides a minimalistic environment for testing
- Is ideal for integration tests and local development
- Requires minimal setup and configuration

### Swarm-based Testing

The swarm-based testing approach leverages and extends Diem's original Forge::Swarm tools to:

- Run comprehensive end-to-end tests
- Allow introspection into node states
- Support manipulating nodes for specific test scenarios
- Provide detailed test reporting

## Why Not Kubernetes?

While vendor code offers Forge::K8 (Kubernetes-based testing), OL has chosen not to use Kubernetes for container-level abstraction for several reasons:

- Adds unnecessary complexity and instrumentation
- Creates tight coupling with vendor release binaries
- Doesn't align with OL's "production binaries only" philosophy
- Swarm already provides sufficient instrumentation for node introspection

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust toolchain
- Libra Framework repositories

### For Swarm
Swarm starts one or more processes of the `libra` production bin.
Requires that you have a `libra` binary built and
the envvar DIEM_FORGE_NODE_BIN_PATH pointed to it
e.g. DIEM_FORGE_NODE_BIN_PATH=$HOME/.cargo/bin/libra

### Basic Usage

```bash
# Configure a new virgin testnet
libra testnet configure

# Start a Libra Smoke swarm-based virgin testnet
libra testnet smoke

# Create a twin of mainnet for testing
libra testnet --twin-epoch-restore=344 smoke

```

## Configuration

The configuration step generates all necessary files for either virgin or twin mode. This includes:

- Genesis transaction
- Validator configurations
- Network parameters
- Account setup

In twin mode, the configuration is derived from a mainnet snapshot to ensure identical state and behavior.

## Use Cases

### Virgin Mode Testing

- Validating new features from a clean state
- Testing genesis procedures
- Baseline performance testing
- Initial network setup validation

### Twin Mode Testing

- Validating upgrades against production-like data
- Testing migration scripts
- Analyzing behavior with real-world state
- Debugging production issues in a controlled environment

## Extending the Tools

The testnet tools are designed to be extensible. Common extensions include:

- Custom test scenarios
- Specialized network configurations
- Automated test suites
- Performance benchmarking

## Contributing

Contributions to improve the testing tools are welcome. Please follow the standard contribution process for the Libra Framework.
