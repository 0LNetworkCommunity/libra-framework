PK0 = 0xc2d3ac2bdb0d7a009b1d593afbfd0a83ba2301fea23ef1bfb1cb1e803bcc9474
PK1 = 0x200217b12371c0ff25b629349623bca45cea9d7096ff8a35a95c41f64c70e171
PK2 = 0x167d3c3e2196d12fd9d53fcceb7e27b285ac9ba9f2d9edd04c8c5dd4bc2e0a23
UPGRADE_SCRIPT = /root/upgrade-six/1-libra-framework/
API_URL = http://127.0.0.1:44949
PROPOSAL_ID = 6
TMP_DIR = /tmp/.tmpK3MJpH/

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
