
## Genesis Instructions
(WIP)

### Testnet

1. You'll need an empty repo to be the "coordination" repo for all node registration data. See for example `https://github.com/0o-de-lally/a-genesis`

1a. You can copy the layout.yaml file. Later you will edit this file with the list of users participating in genesis.


2. Configure the nodes

2a. Nodes will need to have a number of tools installed in order to compile. This script *might*  work for your distro `https://github.com/0o-de-lally/libra/blob/v6/ol/util/setup.sh`

```
# targeting ubuntu
sudo apt update
sudo apt install -y git tmux jq build-essential cmake clang llvm libgmp-dev pkg-config libssl-dev lld

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
```

3. checkout and build `libra-v7`

You'll want to do "release" builds of everything. Grab a cup.

Pro tip: double check your system installs. If you missed any of the system installs above, you'll be waiting a long time to find out.

```
git clone git@github.com:0o-de-lally/libra-v7.git

cargo build --release -p libra -p libra-genesis-tools -p libra-framework
```


