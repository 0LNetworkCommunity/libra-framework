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
	rescue hard-fork --db-path ${DB_PATH} $(foreach val, ${DEBUG_VALS},--debug-vals $(val)) --account-file ${ACCOUNT_FILE}

hard-fork-commit:
	rescue bootstrap --db-path ${DB_PATH} -g ${DB_PATH}/hard_fork.blob --commit -w ${WAYPOINT}

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


download-db:
	@echo THIS IS A BAD PRACTICE
	@echo you should be starting a fullnode instead
	@echo or using a snapshot from https://github.com/0LNetworkCommunity/epoch-archive-mainnet
	mkdir -p ~/0L_upgrade_81/
	cd ~/0L_upgrade_81/
	pip install gdown
	rm -f ~/0L_upgrade_81/db-33254886.tar.gz
	gdown 1AinUVYZknPM6q__W7vqo4xbhzkgiEF82 -O ~/0L_upgrade_81/db-33254886.tar.gz
	tar -xvf db-33254886.tar.gz
	rm -rf ${DB_PATH}
	mv ~/0L_upgrade_81/home/sirouk/.libra/data-2024-03-14-045027/db ${DB_PATH}
