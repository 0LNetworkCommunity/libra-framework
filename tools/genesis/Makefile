# Makefile for testnet

ifndef TARGET_SUPPLY
TARGET_SUPPLY = 100000000000 # 100 billion
endif

ifndef FUTURE_USES
FUTURE_USES = 0.70
endif

ifndef RECOVERY_FILE
RECOVERY_FILE = sample_end_user_single
# RECOVERY_FILE = sample_export_recovery
# RECOVERY_FILE = sample_rotated_auth
endif

ifndef CHAIN
CHAIN = testnet
endif


genesis:
	cargo r -- --org-github  0LNetworkCommunity \
	--name-github genesis-smoke \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	genesis \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow  2B0E8325DEA5BE93D856CFDE2D0CBA12


wizard:
	cargo r -- --org-github  0LNetworkCommunity \
	--name-github genesis-smoke \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	wizard \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow  2B0E8325DEA5BE93D856CFDE2D0CBA12

stdlib:
	cargo r -p libra-framework -- release


testnet: stdlib test-genesis test-node

test-node:
	cargo r -p libra -- node

ifndef ALICE_IP
ALICE_IP = 134.209.32.159
endif

ifndef BOB_IP
BOB_IP = 174.138.92.116
endif

ifndef CAROL_IP
CAROL_IP = 174.138.92.121
endif

test-genesis:
	LIBRA_CI=1 cargo r -- \
	-c ${CHAIN} testnet \
	-m ${PERSONA} \
	-i ${ALICE} \
	-i ${BOB_IP} \
	-i ${CAROL_IP} \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow  2B0E8325DEA5BE93D856CFDE2D0CBA12
