# NOTE: you'll need to have a db of a fullnode already synced
# twin tool will make a copy of this
# THE DB WILL NOT BE WRITTEN TO
DB_DIR=$$HOME/.libra/data/db
UPGRADE_SCRIPT = $$HOME/upgrade-six/1-libra-framework/
PROPOSAL_ID = 6
TMP_DIR = /tmp/.tmpCu3Rxh/

# start twin with three validators
start-twin:
	cargo run -p libra-twin-tests -- -d ${DB_DIR} -c 3


upgrade: tx-propose tx-vote tx-resolve

tx-propose:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml governance propose -d ${UPGRADE_SCRIPT} -m https://tbd.com

tx-vote:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}
	libra txs -c ${TMP_DIR}/1/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}
	libra txs -c ${TMP_DIR}/2/libra-cli-config.yaml governance vote -i ${PROPOSAL_ID}

tx-resolve:
	libra txs --url ${API_URL} -t  ${PK0} --tx-profile critical governance resolve -i ${PROPOSAL_ID} -d ${UPGRADE_SCRIPT}

tx-epoch:
	libra txs --url ${API_URL} -t  ${PK0} governance epoch-boundary

view-state:
	libra query -c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::diem_governance::get_proposal_state -a ${PROPOSAL_ID}

view-resolve:
	libra query -c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::diem_governance::get_can_resolve -a ${PROPOSAL_ID}

view-vals:
	libra query-c ${TMP_DIR}/0/libra-cli-config.yaml view -f 0x1::stake::get_current_validators


######## OTHER
debug-keys:
	cat ${TMP_DIR}/0/private-identity.yaml
	cat ${TMP_DIR}/1/private-identity.yaml
	cat ${TMP_DIR}/2/private-identity.yaml

help-tx-bid-shuffle:
	libra txs -c ${TMP_DIR}/0/libra-cli-config.yaml validator pof -b 0.3 -e 1000
	libra txs -c ${TMP_DIR}/1/libra-cli-config.yaml validator pof -b 0.4 -e 1000
	libra txs -c ${TMP_DIR}/2/libra-cli-config.yaml validator pof -b 0.5 -e 1000
