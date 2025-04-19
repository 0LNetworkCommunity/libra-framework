TWIN_JSON := ./twin.json
FRAMEWORK := ../../framework
DB_PATH := ~/.libra/db_370

framework:
	cd ${FRAMEWORK} && libra move framework release

twin:
	libra ops testnet -f ${FRAMEWORK}/releases/head.mrb -r ${DB_PATH} --json-file ${TWIN_JSON} smoke

parse-jq:
	$(eval APP_CFG=$(shell jq -r '.[0].app_cfg_path' ${TWIN_JSON}))
	@echo "APP_CFG set to: ${APP_CFG}"

# Helper function to set API endpoint environment variables for a specific node
set-api-endpoint:
	$(eval NODE_INDEX ?= 0)
	$(eval API_HOST=$(shell jq -r '.[${NODE_INDEX}].api_endpoint.host' ${TWIN_JSON}))
	$(eval API_PORT=$(shell jq -r '.[${NODE_INDEX}].api_endpoint.port' ${TWIN_JSON}))
	$(eval export LIBRA_API_HOST=${API_HOST})
	$(eval export LIBRA_API_PORT=${API_PORT})
	@echo "API endpoint set to: ${API_HOST}:${API_PORT} (for node ${NODE_INDEX})"
	@echo "Environment variables LIBRA_API_HOST and LIBRA_API_PORT are now set."

# Helper to set API endpoint and show the full URL
print-api-url: set-api-endpoint
	@echo "Full API URL: http://${LIBRA_API_HOST}:${LIBRA_API_PORT}"

grep-debug:
	$(eval LOG_DIR=$(shell jq -r '.[0].data_dir' ${TWIN_JSON})/log)
	@echo "Searching for EPOCH in: ${LOG_DIR}"
	grep -e "\[debug\]" ${LOG_DIR}

####### TRANSACTIONS

send-human: parse-jq
	libra txs -c ${APP_CFG} user human-founder

send-epoch: parse-jq
	libra txs -c ${APP_CFG} governance epoch-boundary

#### QUERIES

balance: parse-jq
	libra query -c ${APP_CFG} balance ${ADDR}

supply: parse-jq
	libra query -c ${APP_CFG} view -f 0x1::supply::get_stats
	libra query -c ${APP_CFG} view -f 0x1::slow_wallet::get_slow_supply

# New target that uses the API endpoint from twin.json
resources: set-api-endpoint
	curl http://${LIBRA_API_HOST}:${LIBRA_API_PORT}/v1/accounts/${ADDR}/resources > ${ADDR}-resources.json

vouch-score: parse-jq
	libra query -c ${APP_CFG} view -f 0x1::page_rank_lazy::get_cached_score -a ${ADDR}

cw-status: parse-jq
	libra query -c ${APP_CFG} view -f 0x1::donor_voice_reauth::is_authorized -a ${CW}
	libra query -c ${APP_CFG} resource -r 0x1::donor_voice_reauth::DonorAuthorized ${CW}
	libra query -c ${APP_CFG} view -f 0x1::donor_voice_governance::is_reauth_proposed -a ${CW}
	libra query -c ${APP_CFG} view -f 0x1::donor_voice_governance::get_reauth_tally -a ${CW}

cw-auth-vote: parse-jq
	libra txs generate-transaction -f 0x1::donor_voice_txs::vote_reauth_tx -a ${CW}

	# libra txs community reauthorize --community-wallet ${CW}

query-roots:  parse-jq
	libra query -c ${APP_CFG} view -f 0x1::root_of_trust::get_current_roots_at_registry -a 0x1


##### upgrades
upgrade-payload:
	libra move framework upgrade -o $$HOME/libra-upgrade  --framework-local-dir ${FRAMEWORK} --core-modules libra-framework --danger-force-upgrade

upgrade-propose: parse-jq
	libra txs -c ${APP_CFG} governance propose --proposal-script-dir $$HOME/libra-upgrade/1-libra-framework --metadata-url http://allyourbase.com

upgrade-vote: parse-jq
	libra txs -c ${APP_CFG} governance vote --proposal-id ${PROPOSAL_ID}
	# get next validator
	$(eval APP_CFG=$(shell jq -r '.[1].app_cfg_path' ${TWIN_JSON}))
	libra txs -c ${APP_CFG} user human-founder
	libra txs -c ${APP_CFG} governance vote --proposal-id ${PROPOSAL_ID}
	libra txs -c ${APP_CFG} --tx-profile critical governance resolve --proposal-id ${PROPOSAL_ID} --proposal-script-dir $$HOME/libra-upgrade/1-libra-framework
