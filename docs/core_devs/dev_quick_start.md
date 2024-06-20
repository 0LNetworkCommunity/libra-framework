# Libra Move Dev Quick Start
## TL;DR


### If you are running Move tests:
* You must install `libra` cli tool to your PATH.

```
# in this repo
cargo build --release -p libra


# copy to a dir in your PATH
cp ./target/release/libra ~/.cargo/bin
# you may need to make it executable
chmod +x ~/.cargo/bin/libra

# run framework tests with
cd ./framework/libra-framework
libra move framework test

# fun formal verification with
libra move framework prove
```

### If you are running e2e smoke tests:
* You need our fork of `diem-node` before working on `libra-framework`
* compile `diem-node` to `$HOME/.cargo/bin`

```
git clone https://github.com/0LNetworkCommunity/diem -b release --single-branch
export RUST_DIEM_COIN_MODULE="libra_coin"
export RUST_DIEM_COIN_NAME="LibraCoin"
cd diem
cargo build --profile cli -p diem-node --target-dir ~/.cargo/bin

# make it executable
chmod +x ~/.cargo/bin/diem-node
```
* export these env vars in your dev env, `~/.bashrc` or `~/.zshrc`

```
export RUST_MIN_STACK=104857600
export DIEM_FORGE_NODE_BIN_PATH="$HOME/.cargo/bin/diem-node"
```
## Set up environment

You should have two repos that you are working with. This one `libra-framework`, as well as `diem`. We'll need to build some executables from diem and install them on your dev machine.

### Get the DIEM dependencies

You need our fork of diem before working on `libra-framework`
```
git clone https://github.com/0LNetworkCommunity/diem -b release --single-branch
```
### check env
This assumes that you have a `~/.cargo/bin` which is added to your environment's $PATH.

### build executables
You want to build the `diem-node` (for smoke tests only).

There are two environment variables that are needed to use the correct Coin for
diem-node instead of a generic.

`export RUST_DIEM_COIN_MODULE: "libra_coin"`
`export RUST_DIEM_COIN_NAME: "LibraCoin"`

Note that the `--profile cli` compilation profile makes for much smaller binaries (e.g. `diem-node` goes from about 2GB to 30MB).

```
# env variables needed for compilation
export RUST_DIEM_COIN_MODULE="libra_coin"
export RUST_DIEM_COIN_NAME="LibraCoin"

# build it
cargo build --profile cli -p diem-node --target-dir ~/.cargo/bin
# see you tomorrow (welcome to rust).

# next day, make it executable.
chmod +x ~/.cargo/diem-node
```

Just check those executables appear in your path.
`which diem-node`

Now you can run commands as below.
# Running Move unit tests
Change into a Move project dir (i.e., the directory with a Move.toml).

`diem move test`


optionally with filters:

`diem move test -f`

## Build a libra framework release for smoke tests (head.mrb)


```
cd ./framework
cargo run release

```

Your release will be in `./releases/head.mrb`, you will need this for genesis and smoketests.

Note for smoke tests: you must regenerate the .mrb file, EVERYTIME YOU MAKE A CHANGE TO CORE MOVE CODE. Otherwise your tests will be against the old code

## running smoke tests

Do it yourself:
Make sure you are in the root of the project.

Note: there is an issue with the rust default stack size for tests which involve compiling, and then starting a local testnet

```
cd ./smoke-tests
export RUST_MIN_STACK=104857600
export DIEM_FORGE_NODE_BIN_PATH="$HOME/.cargo/bin/diem-node"
cargo test

```

## Troubleshooting

### 1. Building diem-node

#### Issue
If you encounter the following error:
`error[E0599]: no method named disable_lifo_slot found for mutable reference &mut tokio::runtime::Builder in the current scope`

#### Solution
You can resolve this issue by building with the following flag:

```
RUSTFLAGS="--cfg tokio_unstable" cargo build --profile cli -p diem-node --target-dir ~/.cargo/bin
```

This flag enables the unstable features required by the tokio runtime.
