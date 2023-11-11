# Makefile for testnet

ifndef TARGET_SUPPLY
TARGET_SUPPLY = 100000000000 # 100 billion
endif

ifndef FUTURE_USES
FUTURE_USES = 0.70
endif

ifndef RECOVERY_FILE
# RECOVERY_FILE =
RECOVERY_FILE = ./tests/fixtures/v5_recovery.json
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
GIT_REPO = release-v6.9.0-rc.0-genesis-7
endif

install:
	cargo build --release -p libra -p libra-genesis-tools -p libra-framework
	cp ../../target/release/libra* ~/.cargo/bin/

register:
	libra-genesis-tools register --org-github ${GIT_ORG} --name-github ${GIT_REPO}
# have cli tools use localhost
	libra config fix --force-url http://localhost:8080

legacy:
### Fetch Ancestry Data, Snapshot, and Use v5.2 codebase and snapshot to generate recovery.json for seeding v6.9.x state
	sudo rm -Rf ~/libra-recovery && mkdir -p ~/libra-recovery
	wget https://github.com/0LNetworkCommunity/epoch-archive/raw/main/670.tar.gz -O ~/libra-recovery/670.tar.gz
	cd ~/libra-recovery && tar -xvzf 670.tar.gz
	wget https://raw.githubusercontent.com/sirouk/ol-data-extraction/v-6.9.x-ready/assets/data.json -O ~/libra-recovery/v5_ancestry.json

	sudo rm -Rf ~/libra-legacy-v6
	cd ~ && git clone -b v6 https://github.com/0LNetworkCommunity/libra-legacy-v6
	cd ~/libra-legacy-v6/ol/genesis-tools
	cargo r -p ol-genesis-tools -- --export-json ~/libra-recovery/v5_recovery.json --snapshot-path ~/libra-recovery/670/state_ver* --ancestry-file ~/libra-recovery/v5_ancestry.json
	cp -f ~/libra-recovery/v5_recovery.json ~/libra-framework/tools/genesis/tests/fixtures/v5_recovery.json
	md5sum ~/libra-framework/tools/genesis/tests/fixtures/v5_recovery.json"

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
