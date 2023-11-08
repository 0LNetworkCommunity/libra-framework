# Makefile for testnet

ifndef TARGET_SUPPLY
TARGET_SUPPLY = 100000000000 # 100 billion
endif

ifndef FUTURE_USES
FUTURE_USES = 0.70
endif

ifndef RECOVERY_FILE
# RECOVERY_FILE = v5_recovery
RECOVERY_FILE = sample_export_recovery

endif

ifndef YEARS
YEARS = 7
endif

ifndef CHAIN
CHAIN = testnet
endif

ifndef GIT_ORG
GIT_ORG = 0LNetworkCommunity
endif

ifndef GIT_REPO
GIT_REPO = genesis-smoke
endif


genesis: stdlib
	cargo r -- \
	-c ${CHAIN} \
	genesis --org-github ${GIT_ORG} \
	--name-github ${GIT_REPO} \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--years-escrow ${YEARS} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow 2B0E8325DEA5BE93D856CFDE2D0CBA12


wizard:
	cargo r -- 	-c ${CHAIN} \
	wizard --org-github ${GIT_ORG} \
	--name-github ${GIT_REPO} \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--years-escrow ${YEARS} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow 2B0E8325DEA5BE93D856CFDE2D0CBA12

stdlib:
	cargo r -p libra-framework -- release

git-fetch:
	@echo WILL RESET GIT AND PULL LATEST COMMIT
	git reset --hard && git pull -f

testnet: git-fetch stdlib test-genesis node

test-node:
	cargo r -p libra -- node

node:
	diem-node -f ~/.libra/validator.yaml

ifndef ALICE_IP
ALICE_IP = 134.209.32.159
endif

ifndef BOB_IP
BOB_IP = 165.22.44.147
endif

ifndef CAROL_IP
CAROL_IP = 165.22.34.98
endif

test-genesis:
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
