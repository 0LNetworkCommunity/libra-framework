
TARGET_SUPPLY = 10000000000 # 10 billion
FUTURE_USES = 0.70
# RECOVERY_FILE = sample_end_user_single
RECOVERY_FILE = sample_export_recovery

genesis:
	cargo r -- --org-github  0o-de-lally \
	--name-github a-genesis \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	genesis \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow  2B0E8325DEA5BE93D856CFDE2D0CBA12


wizard:
	cargo r -- --org-github  0o-de-lally \
	--name-github a-genesis \
	--local-framework \
	--json-legacy ./tests/fixtures/${RECOVERY_FILE}.json \
	wizard \
	--target-supply ${TARGET_SUPPLY} \
	--target-future-uses ${FUTURE_USES} \
	--map-dd-to-slow 3A6C51A0B786D644590E8A21591FA8E2 \
	--map-dd-to-slow  2B0E8325DEA5BE93D856CFDE2D0CBA12