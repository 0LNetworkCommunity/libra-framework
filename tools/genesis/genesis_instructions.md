
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
cd libra-v7
git checkout release-0.6.9

cargo build --release -p libra -p libra-genesis-tools
```

4. Get your Github app api token


4a. Follow the github instructions: `https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-personal-access-token-classic`

You'll want to choose the `repo permissions` setting.

4b You'll need a directory at `$HOME/.libra`. Place your github api token in `github_token.txt` under that directory.

5. Do the genesis ceremony.
You'll use the wizard for both configuring, registering, and building the genesis transaction.

You'll input the name of the github repo (`--org-github` and `--name-github `) being used to coordinate. 
```
./target/release/libra-genesis-tools  --org-github <ORG_GITHUB> --name-github <NAME_GITHUB> register 
```

6. Coordinator: merge pull requests.

The owner of the coordinator repo should merge the pull requests the registrants made to the repo.

7. Run the genesis transaction builder with `libra-genesis-tools genesis`

You'll use the same github arguments as above plus two more. You'll be using a local copy of the move framework (`--local-framework`). Last, you'll tell the wizard which DB backup file to use to migrate state from the previous network (`--json-legacy`). 

For the legacy JSON you can use the test example: `tools/genesis/tests/fixtures/sample_export_recovery.json`

```
./target/release/libra-genesis-tools  --org-github <ORG_GITHUB> --name-github <NAME_GITHUB> --local-framework --json-legacy <PATH_TO_JSON> genesis
```

8. Check your files
You should have a `genesis/genesis.blob` file now in `$HOME/libra` plus a `validator.yaml`.

9. Start your node!

```
libra node --config-path ~/.libra/validator.yaml
```
### Troubleshooting

1. I made changes to a .move file

You'll need to rebuild the framework and its generated code.
```
cd framework/
cargo r --release release
# yes two releases in there.
```


