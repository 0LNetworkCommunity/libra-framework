TWIN_JSON = ./twin.json
FRAMEWORK = ../../framework
DB_PATH = ~/.libra/db_365

framework:
	cd ${FRAMEWORK} && libra move framework release

twin:
	cargo r -p libra -- ops testnet -f ${FRAMEWORK}/releases/head.mrb -r ${DB_PATH} --json-file ${TWIN_JSON} smoke

parse-jq:
	$(eval APP_CFG=$(shell jq -r '.[0].app_cfg_path' ${TWIN_JSON}))
	@echo "APP_CFG set to: ${APP_CFG}"

grep-debug:
	$(eval LOG_DIR=$(shell jq -r '.[0].data_dir' ${TWIN_JSON})/log)
	@echo "Searching for EPOCH in: ${LOG_DIR}"
	grep -e "\[debug\]" ${LOG_DIR}

####### TRANSACTIONS

send-human: parse-jq
	cargo r -p libra -- txs -c ${APP_CFG} user human-founder

send-epoch: parse-jq
	cargo r -p libra -- txs -c ${APP_CFG} governance epoch-boundary

#### QUERIES
query-roots:  parse-jq
	libra query -c ${APP_CFG} view -f 0x1::root_of_trust::get_current_roots_at_registry -a 0x1
