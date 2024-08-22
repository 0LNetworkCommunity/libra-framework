PK0 = 0x2c1495930c667a8d184b591c29fe99340cc0f19ae0668f6d5b616947f6fb8d6c
PK1 = 0xc6ee87812a58cdce6917e544d6bb3da6e9d8bd81b6265894259744d37f33c4d0
PK2 = 0xcbaa77abda240ee14d271a97be8eb45049918f19a63b86746021990d5efd6f73
UPGRADE_SCRIPT = /root/upgrade-six/1-libra-framework/
API_URL = http://127.0.0.1:42199
PROPOSAL_ID = 6
TMP_DIR = /tmp/.tmpCu3Rxh/

tx-propose:
	libra txs --url ${API_URL} -t ${PK0} governance propose -d ${UPGRADE_SCRIPT} -m https://tbd.com

tx-vote:
	libra txs --url ${API_URL} -t ${PK0} governance vote -i ${PROPOSAL_ID}
	libra txs --url ${API_URL} -t ${PK1} governance vote -i ${PROPOSAL_ID}
	libra txs --url ${API_URL} -t ${PK2} governance vote -i ${PROPOSAL_ID}

tx-bid:
	libra txs --url ${API_URL} -t ${PK0} validator pof -b 0.3 -e 1000
	libra txs --url ${API_URL} -t ${PK1} validator pof -b 0.4 -e 1000
	libra txs --url ${API_URL} -t ${PK2} validator pof -b 0.5 -e 1000

tx-resolve:
	libra txs --url ${API_URL} -t  ${PK0} --tx-profile critical governance resolve -i ${PROPOSAL_ID} -d ${UPGRADE_SCRIPT}

tx-epoch:
	libra txs --url ${API_URL} -t  ${PK0} governance epoch-boundary

view-state:
	libra query view -f 0x1::diem_governance::get_proposal_state -a ${PROPOSAL_ID}

view-resolve:
	libra query view -f 0x1::diem_governance::get_can_resolve -a ${PROPOSAL_ID}

view-vals:
	libra query view -f 0x1::stake::get_current_validators

show-keys:
	cat ${TMP_DIR}/0/private-identity.yaml
	cat ${TMP_DIR}/1/private-identity.yaml
	cat ${TMP_DIR}/2/private-identity.yaml

upgrade: tx-propose tx-vote tx-resolve
