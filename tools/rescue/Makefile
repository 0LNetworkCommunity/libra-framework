SHELL=/usr/bin/env bash



ifndef SOURCE_PATH
SOURCE_PATH=~/libra-framework
endif

ifndef LIBRA_PATH
LIBRA_PATH=~/.libra
endif


ifndef DATA_PATH
DATA_PATH=${LIBRA_PATH}/data
endif

ifndef DB_PATH
DB_PATH=${DATA_PATH}/db
endif


ifndef VALS_FILE
VALS_FILE=${SOURCE_PATH}/tools/rescue/fixtures/fork/validators.json
endif

ifndef ACCOUNT_FILE
ACCOUNT_FILE=${SOURCE_PATH}/tools/rescue/fixtures/fork/sample_fork_user.json
endif


ifndef WAYPOINT
WAYPOINT=""
endif



check:
	git log -n 1 --pretty=format:'%H'
	@echo "Make sure to verify the git hash above with your peers!"
	@echo "Forked validator set will consist of the following:"
	@cat ${VALS_FILE}



######## BUILD ########
bins:
	cd ${SOURCE_PATH}/framework && \
	cargo clean && \
	cargo r release && \
	cargo build --release -p rescue

install:
	sudo cp -f ${SOURCE_PATH}/target/release/rescue ~/.cargo/bin/
	@echo "The following should match: ${HOME}/.cargo/bin/rescue"
	@which rescue



######## WRITESETS ########
hard-fork-blob:
	rescue hard-fork --db-dir ${DB_PATH} --validators-file ${VALS_FILE} --account-file ${ACCOUNT_FILE} --staging-mode

hard-fork-commit:
	rescue bootstrap --db-dir ${DB_PATH} --genesis-txn-file ${DB_PATH}/hard_fork.blob --waypoint-to-verify ${WAYPOINT} --commit
	@$(MAKE) update-waypoint


framework-upgrade-blob:
	rescue rescue-tx --db-dir ${DB_PATH} --framework-upgrade

rescue-commit:
	rescue bootstrap --db-dir ${DB_PATH} --genesis-txn-file ${DB_PATH}/rescue.blob --waypoint-to-verify ${WAYPOINT} --commit
	@$(MAKE) update-waypoint


update-waypoint:
	@echo "Updating waypoint.txt with value:"
	echo ${WAYPOINT} > ${LIBRA_PATH}/genesis/waypoint.txt
	@cat ${LIBRA_PATH}/genesis/waypoint.txt



######## UTILS ########
backup-db:
	-pkill -f libra
	rm -f ${DATA_PATH}/secure-data.json

	@backup_dir=$(DATA_PATH)-$$(date +"%Y-%m-%d-%H%M%S"); \
	if [ -d "$$backup_dir" ]; then \
		echo "Backup already exists for $$backup_dir, skipping backup."; \
	else \
		cp -r $(DATA_PATH) $$backup_dir; \
		echo "Backup successful: $$backup_dir"; \
	fi

fetch-db:
	@echo "This will be replaced by either starting a fullnode or using a snapshot from the epoch-archive-mainnet repo."
	@echo "For simplicity, we will use the db from the machine that maintains the epoch-archive-mainnet repo."
	
	@echo "Fetching database..."
	rm -rf ${DB_PATH}
	wget https://storage.googleapis.com/0l-network/db-33254886.tar.gz -O db-33254886.tar.gz
	tar -xvzf db-33254886.tar.gz --strip-components=4 home/sirouk/.libra/data-2024-03-14-045027/db
	mv ./db ${DB_PATH}
	rm -f db-33254886.tar.gz


test-api:
	curl localhost:8080/v1/accounts/0x9A710919B1A1E67EDA335269C0085C91 | jq
	curl localhost:8080/v1/accounts/0x1edc494090644bf6fcdd934b3cabace0e1d06fc9f82328b47a7636cf8b49f37a | jq
	curl localhost:8080/v1/accounts/0x0407d4d486fdc4e796504135e545be77 | jq
	curl localhost:8080/v1/accounts/0x0407d4d486fdc4e796504135e545be77/resources | jq
	curl localhost:8080/v1/accounts/0xb28117ef2e8f104b80ff54d13e375cbe/resources | jq
