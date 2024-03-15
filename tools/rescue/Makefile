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


backup:

	-pkill -f libra
	rm -f ${DATA_PATH}/secure-data.json

	@backup_dir=$(DATA_PATH)-$$(date +"%Y-%m-%d-%H%M%S"); \
	if [ -d "$$backup_dir" ]; then \
		echo "Backup already exists for $$backup_dir, skipping backup."; \
	else \
		cp -r $(DATA_PATH) $$backup_dir; \
		echo "Backup successful: $$backup_dir"; \
	fi
	
	
fetch-snapshot:

	mkdir -p ~/0L_upgrade_81/ && \
	cd ~/0L_upgrade_81/ && \
	pip install gdown && \
	rm -f ~/0L_upgrade_81/db-33254886.tar.gz && \
	gdown 1AinUVYZknPM6q__W7vqo4xbhzkgiEF82 -O ~/0L_upgrade_81/db-33254886.tar.gz && \
	tar -xvf db-33254886.tar.gz && \
	rm -rf ${DB_PATH} && \
	mv ~/0L_upgrade_81/home/sirouk/.libra/data-2024-03-14-045027/db ${DB_PATH}


build:

	cd ${SOURCE_PATH} && \
	git fetch --all && \
	git checkout cyber-knife-test && \
	git pull && \
	cargo build --release -p rescue && \
	cd ${SOURCE_PATH}/framework && \
	cargo r release && \
	git log -n 1 --pretty=format:'%H'

	echo "Make sure to verify the git hash above with your peers!"


write-valset:

	cd ${SOURCE_PATH}/target/release/ && \
	./rescue rescue-tx --data-path $(DB_PATH) --framework-upgrade $(foreach val,$(DEBUG_VALS),--debug-vals $(val))


bootstrap-valset:

	cd ${SOURCE_PATH}/target/release/ && \
	./rescue bootstrap ${DB_PATH} -g ${DB_PATH}/rescue.blob --waypoint-to-verify ${WAYPOINT} --commit


upgrade-framework:

	cd ${SOURCE_PATH}/target/release/ && \
	./rescue rescue-tx --data-path ${DB_PATH} --framework-upgrade


bootstrap-framework:

	cd ${SOURCE_PATH}/target/release/ && \
	./rescue bootstrap --genesis-txn-file ${DB_PATH}/rescue.blob --waypoint-to-verify ${WAYPOINT} --commit ${DB_PATH} && \
	./rescue hard-fork --db-dir ${DB_PATH} --account-file ${SOURCE_PATH}/tools/rescue/fixtures/fork/sample_fork_user.json && \
	echo ${WAYPOINT} > ${LIBRA_PATH}/genesis/waypoint.txt

	echo "Prepare to start your node"

