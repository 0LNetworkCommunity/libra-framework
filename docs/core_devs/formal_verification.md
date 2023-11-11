# Move Formal Verification

The Move language includes a syntax for annotating code with `spec`
specifications which can be "formally verified". The syntax is purpose built for
Move. The stack behind the verification is based on `boogie`.

You will need to install many dependencies for the Move test tooling as well as
boogie system libraries.

## Quick Start

1) get the libra cli with standard Move tools

Compile the `libra` cli app, and have it in your executable PATH.
```
cargo b --profile cli -p libra
cp target/cli/libra ~/.cargo/bin
chmod +x ~/.cargo/bin/libra

# check it runs
libra -h
# this is the subcommand for the formal verification
libra move prove -h
```

2) Install the Boogie dependencies

The `dev_setup.sh` can be run with these options:

> -y install or update Move Prover tools: z3, cvc5, dotnet, boogie

> -p update ${HOME}/.profile or ./bashrc

```
# run the installer
bash util/dev_setup.sh -yp

# you may need to restart your shell, after env variables are set
# or .bashrc, or .zshrc

bash ~/.profile

```

Whatever terminal shell (or .zshrc) you are using you should check that these variable are exported:
```
export DOTNET_ROOT="$HOME/.dotnet"
export Z3_EXE="$HOME/bin/z3"
export CVC5_EXE="$HOME/bin/cvc5"
export BOOGIE_EXE="$HOME/.dotnet/tools/boogie"
export SOLC_EXE="$HOME/bin/solc"
```

3) check it all works
you'll be running formal verification specs against `libra-framework` move
source.

So test it on something we know to work
```
cd framework/libra-framework

# test the guid.move module
libra move prove -f guid
```

If you get a response without errors similar to the message below you are ready to
start.
```
[INFO] 0.903s build, 2.585s trafo, 0.019s gen, 1.313s verify, total 4.820s
```

## Troubleshooting

1) check then env vars were set up `echo $BOOGIE_EXE`
2) check that all the system libraries were installed `ls $HOME/.dotnet/tools/boogie`

# Goals

Formal verification priority for Libra should mainly check that the network does
not halt in operations conducted by the VM.

1. epoch_boundary.move
Should never halt on reconfigure()
THough there are many downstream modules and functions from here, the largest
ones being:

    a. stake.move

    b. proof_of_fee.move

2. coin.move
  a. Transactions by VM should not halt

3. slow_wallet.move
  a. no transactions should bypass the slow wallet tracker. If there is a slow
  wallet struct, a trasaction should always alter it.
  b. no account, not even the VM can withdraw above the unlocked limit.
  c. the unlocked limit cannot be larger than the total.