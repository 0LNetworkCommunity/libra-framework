# Examples of how to use the storage tool.
# uses example fixture files located here in ./tools/storage/fixtures
# try this: `make -f example.makefile restore`

# you need to use the absolute path of the snapshot files.
ABS_PATH?=`cd "$1"; pwd`
BACKUP_DIR= ${ABS_PATH}/fixtures/v7/
OUTPUT_DB_PATH = ${ABS_PATH}/test_db

# For the epoch restore
EPOCH_MANIFEST = ${BACKUP_DIR}epoch_ending_116-.be9b/epoch_ending.manifest

# For the snapshot restore
STATE_MANIFEST = ${BACKUP_DIR}state_epoch_116_ver_38180075.05af/state.manifest

restore: restore-epoch restore-snapshot

restore-epoch:
	libra ops storage db restore oneoff epoch-ending --epoch-ending-manifest ${EPOCH_MANIFEST} --target-db-dir ${OUTPUT_DB_PATH} --local-fs-dir ${BACKUP_DIR}

restore-snapshot:
	libra ops storage db restore oneoff state-snapshot --state-manifest ${STATE_MANIFEST} --target-db-dir ${OUTPUT_DB_PATH} --local-fs-dir ${BACKUP_DIR} --restore-mode default --state-into-version 1

echo:
	@echo ${ABS_PATH}
	@echo ${BACKUP_DIR}
	@echo ${EPOCH_MANIFEST}
	@echo ${STATE_MANIFEST}
