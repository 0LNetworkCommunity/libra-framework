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

ifndef WAYPOINT
WAYPOINT=""
endif

ifndef DEBUG_VALS
DEBUG_VALS=""
endif

check:
	git log -n 1 --pretty=format:'%H'
	@echo "Make sure to verify the git hash above with your peers!"
	@echo $(foreach val, ${DEBUG_VALS},--debug-vals $(val))

######## BUILD ########
bins:
	cd ${SOURCE_PATH}/framework && cargo r release
	cargo build --release -p rescue

install:
	cp ${SOURCE_PATH}/target/release/rescue ~/.cargo/bin/
	@which rescue

######## WRITESETS ########

hard-fork-blob:
	rescue hard-fork --db-dir ${DB_PATH} --validators-file ${VALS_FILE} --account-file ${ACCOUNT_FILE}

hard-fork-commit:
	rescue bootstrap --db-dir ${DB_PATH} -g ${DB_PATH}/hard_fork.blob --commit -w ${WAYPOINT}

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


bad-bad-download-db:
	@echo THIS IS A BAD PRACTICE, please stop doing it
	@echo you should be starting a fullnode instead
	@echo or using a snapshot from https://github.com/0LNetworkCommunity/epoch-archive-mainnet
	pip install gdown
	mkdir -p ${DATA_PATH}/0L_upgrade_81/
	gdown 1AinUVYZknPM6q__W7vqo4xbhzkgiEF82 -O ${DATA_PATH}/db-33254886.tar.gz
	tar -xvf ${DATA_PATH}/db-33254886.tar.gz

test-api:
	curl localhost:8080/v1/accounts/0x9A710919B1A1E67EDA335269C0085C91 | jq
	curl localhost:8080/v1/accounts/0x1edc494090644bf6fcdd934b3cabace0e1d06fc9f82328b47a7636cf8b49f37a | jq
	curl localhost:8080/v1/accounts/0x0407d4d486fdc4e796504135e545be77 | jq
	curl localhost:8080/v1/accounts/0x0407d4d486fdc4e796504135e545be77/resources | jq
