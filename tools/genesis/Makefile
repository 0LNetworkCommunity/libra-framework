SHELL=/usr/bin/env bash

ifndef BINS_PATH
BINS_PATH = $$HOME/.cargo/bin
endif

ifndef SOURCE_PATH
SOURCE_PATH = $$HOME/libra-framework
endif

ifndef DATA_PATH
DATA_PATH = $$HOME/.libra
endif

ifndef TARGET_SUPPLY
TARGET_SUPPLY = 100000000000 # 100 billion
endif

ifndef FUTURE_USES
FUTURE_USES = 0.70
endif

ifndef RECOVERY_FILE
RECOVERY_FILE = ${DATA_PATH}/libra-recovery/v5_recovery.json
endif

ifndef YEARS
YEARS = 7
endif

ifndef CHAIN
CHAIN = mainnet
endif

ifndef GIT_ORG
GIT_ORG = 0LNetworkCommunity
endif

ifndef GIT_REPO
GIT_REPO=release-v6.9.0-rc.9-genesis
endif

ifndef EPOCH
EPOCH=692
endif

check:
	@echo SOURCE_PATH: ${SOURCE_PATH}
	@echo BINS_PATH: ${BINS_PATH}
	@echo EPOCH: ${EPOCH}

install:
	cd ${SOURCE_PATH} && bash ./util/dev_setup.sh -bt && ${BINS_PATH}/cargo build --release -p libra -p libra-genesis-tools -p libra-framework
# legacy source
	mkdir -p ${DATA_PATH}
	cd ${DATA_PATH} && git clone -b v6 https://github.com/0LNetworkCommunity/libra-legacy-v6
	cp -f ${SOURCE_PATH}/target/release/libra* ${BINS_PATH}/

register:
	libra-genesis-tools register --org-github ${GIT_ORG} --name-github ${GIT_REPO}
# have cli tools use localhost
	libra config fix --force-url http://localhost:8080

legacy: get-legacy recovery

get-legacy:
### Fetch Ancestry Data, Snapshot, and Use v5.2 codebase and snapshot to generate recovery.json for seeding v6.9.x state
	mkdir -p ${DATA_PATH}/libra-recovery | true
	wget https://github.com/0LNetworkCommunity/epoch-archive/raw/main/${EPOCH}.tar.gz -O ${DATA_PATH}/libra-recovery/${EPOCH}.tar.gz
	cd ${DATA_PATH}/libra-recovery && tar -xvzf ${EPOCH}.tar.gz
	wget https://raw.githubusercontent.com/sirouk/ol-data-extraction/v-6.9.x-ready/assets/data.json -O ${DATA_PATH}/libra-recovery/v5_ancestry.json

recovery:
	cd ${DATA_PATH}/libra-legacy-v6/ol/genesis-tools && \
	cargo r -p ol-genesis-tools -- --export-json ${DATA_PATH}/libra-recovery/v5_recovery.json --snapshot-path ${DATA_PATH}/libra-recovery/${EPOCH}/state_ver* --ancestry-file ${DATA_PATH}/libra-recovery/v5_ancestry.json

	md5sum ${RECOVERY_FILE}

genesis: stdlib
	libra-genesis-tools \
	-c ${CHAIN} \
	genesis --org-github ${GIT_ORG} \
	--name-github ${GIT_REPO} \
	--local-framework \
	--json-legacy ${RECOVERY_FILE} \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--years-escrow ${YEARS} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow 2B0E8325DEA5BE93D856CFDE2D0CBA12


stdlib:
	libra-framework release

############ TESTNET HELPERS ################


testnet: testnet-git-fetch testnet-stdlib testnet-genesis testnet-node

testnet-git-fetch:
	@echo WILL RESET GIT AND PULL LATEST COMMIT
	git reset --hard && git pull -f

testnet-stdlib:
	cargo r -p libra-framework -- release

testnet-node:
# assumes you have done `install` above
	libra node

ifndef ALICE_IP
ALICE_IP = 134.209.32.159
endif

ifndef BOB_IP
BOB_IP = 165.22.44.147
endif

ifndef CAROL_IP
CAROL_IP = 165.22.34.98
endif

testnet-genesis:
	LIBRA_CI=1 cargo r -- \
	-c ${CHAIN} testnet \
	-m ${PERSONA} \
	-i ${ALICE_IP} \
	-i ${BOB_IP} \
	-i ${CAROL_IP} \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--years-escrow ${YEARS} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow 2B0E8325DEA5BE93D856CFDE2D0CBA12
