# NOTE: you'll need to have a db of a fullnode already synced
# twin tool will make a copy of this
# THE DB WILL NOT BE WRITTEN TO

# TMP_DIR = /tmp/.tmpCu3Rxh/

ifndef DB_DIR
DB_DIR=$$HOME/.libra/data/db
endif

ifndef UPGRADE_SCRIPT_PATH
UPGRADE_SCRIPT_PATH = $$HOME/upgrade-six/
endif

ifndef FRAMEWORK_SOURCE_PATH
FRAMEWORK_SOURCE_PATH = $$HOME/libra-framework/framework
endif

PROPOSAL_ID = 6

##### INSTRUCTIONS

# 1. use an up to date libra-cli binary
# > cargo build --release -p libra --locked
# > cp target/release/libra ~/.cargo/bin/

# 2. Make sure you have a fullnode that is already syced, you'll need the DB_DIR of it
# if you are starting fresh use:
# > libra config fullnode-init
# > libra node

# 2. compile the new libra-framework MOVE code with:
# > make upgrade-script
# note the defaults for input: FRAMEWORK_SOURCE_PATH and and output: UPGRADE_SCRIPT_PATH

# 3. start a Twin swarm locally
# > make start twin
# NOTE: the output `temp files found at: /tmp/<......> `
# A local Twin of mainnet is now running on your machine.

# 4. export that temp dir to your path
# > export TMP_DIR=/tmp/<......>

# Now your can do transactions as the new random validators
# 5. check validator set
# > make view-vals

# 6. try to tigger epoch
# > make tx-epoch
# NOTE: this usually should fail unless enough time has passed.

# 7. Send the full upgrade e2e
# > make upgrade-ceremony

# 8. check the state of the proposal
# > make view-state

# start twin with three validators
start-twin:
	cargo run -p libra-twin-tests -- -d ${DB_DIR} -c 3


#########  UPGRADE SCRIPT GENERATION
upgrade-script: move-build-framework move-build-script

move-build-framework:
	cd ${FRAMEWORK_SOURCE_PATH} && libra move framework release

move-build-script:
	libra move framework upgrade --core-modules libra-framework --output-dir ${UPGRADE_SCRIPT_PATH} --framework-local-dir ${FRAMEWORK_SOURCE_PATH}

######## EPOCH TRIGGER
tx-epoch:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml governance epoch-boundary


######## UPGRADE TRANSACTIONS
upgrade-ceremony: tx-propose tx-vote tx-resolve

tx-propose:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml governance propose -d ${UPGRADE_SCRIPT_PATH}/1-libra-framework -m https://tbd.com

tx-vote:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}
	libra txs -c ${TMP_DIR}/1/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}
	libra txs -c ${TMP_DIR}/2/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}

tx-resolve:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml --tx-profile critical governance resolve -i ${PROPOSAL_ID} -d ${UPGRADE_SCRIPT_PATH}/1-libra-framework

#### VIEW STATE OF UPGRADE PROPOSALS
view-state:
	libra query -c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::diem_governance::get_proposal_state -a ${PROPOSAL_ID}

view-resolve:
	libra query -c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::diem_governance::get_can_resolve -a ${PROPOSAL_ID}

view-vals:
	libra query -c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::stake::get_current_validators


######## OTHER
debug-keys:
	cat ${TMP_DIR}/0/private-identity.yaml
	cat ${TMP_DIR}/1/private-identity.yaml
	cat ${TMP_DIR}/2/private-identity.yaml

help-tx-bid-shuffle:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml validator pof -b 0.3 -e 1000
	libra txs -c ${TMP_DIR}/1/libra-cli-config.yaml validator pof -b 0.4 -e 1000
	libra txs -c ${TMP_DIR}/2/libra-cli-config.yaml validator pof -b 0.5 -e 1000
