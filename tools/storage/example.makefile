# Examples of how to use the storage tool.
# uses example fixture files located here in ./tools/storage/fixtures
# try this: `make -f example.makefile restore`

# you need to use the absolute path of the snapshot files.
ABS_PATH=`cd "$1"; pwd`
BACKUP_DIR= ${ABS_PATH}/fixtures/v7/
MANIFEST=${BACKUP_DIR}epoch_ending_116-.be9b/epoch_ending.manifest
OUTPUT_DB_PATH = ${ABS_PATH}/test_db

restore:
		libra ops storage db restore oneoff epoch-ending --epoch-ending-manifest ${MANIFEST} --target-db-dir ${OUTPUT_DB_PATH} --local-fs-dir ${BACKUP_DIR}

echo:
	@echo ${ABS_PATH}
	@echo ${BACKUP_DIR}
	@echo ${MANIFEST}
