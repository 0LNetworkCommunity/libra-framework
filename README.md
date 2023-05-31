# 0L

A reference implementation of a neutral replicated state machine. Forked from the Libra/Diem technologies.

## Dev Quick Start

### Set up environment

You should have two repos that you are working with. This one `libra-v7`, as well as `zapatos`. We'll need some executables from zapatos.

#### check env
This assumes that you have a `~/.cargo/bin` which is added to your environment's $PATH.

Export the path to your `zapatos` source, to make this easier. 

`export ZAPATOS="~/path/to/source"`
#### build executables
You want to create a `zapatos` executable so you can run the `move` cli with the framework changes.

You'll want `diem` (cli for move tests), `diem-framework` (framework compiler), `diem-node` (for smoke tests only).

```
cd $ZAPATOS
cargo build --release -p diem-framework -p diem -p diem-node --target-dir ~/.cargo/bin
cd ~/.cargo/bin
mv diem-framework zapatos-framework
mv diem zapatos
```

Just check those executables appear in your path.
`which zapatos`

Now you can run commands as below.
## Run Move Tests

`zapatos move test`


optionally with filters:

`zapatos move test -f`

## Build a release (.mrb)

Make sure you are in the root of `libra-v7`.

`zapatos-framework custom --packages ./ol-framework --rust-bindings "" --output ./ol-framework/releases/head.mrb

Your release will be in head.mrb, you will need this for genesis and smoke tests.

## Run smoke tests

Quickstart: Use the bash script `. ./util/smoke.sh`

Do it yourself:
Make sure you are in the root of `libra-v7`.

```
cd smoke-tests
export MRB_PATH="<path/to>/ol-framework/releases/head.mrb"
export ZAPATOS_BIN_PATH=~/.cargo/bin/

cargo test

```

