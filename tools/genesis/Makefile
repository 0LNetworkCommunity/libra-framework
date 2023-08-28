
TARGET_SUPPLY = 100000000000 # 100 billion
FUTURE_USES = 0.70
RECOVERY_FILE = sample_end_user_single
# RECOVERY_FILE = sample_export_recovery
# RECOVERY_FILE = sample_rotated_auth


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


testnet: stdlib
	LIBRA_CI=1 cargo r testnet -m ${PERSONA} -i 134.209.32.159 -i 174.138.92.116 -i 174.138.92.121
	cargo r -p libra -- node