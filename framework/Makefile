# Prover Tests are WIP
# These are the prover tests that have been written
# and are known to pass
PROVER_TESTS = demo ol_account slow sacred

VENDOR_TESTS = chain_id guid

prove:
	@cd libra-framework && \
	for i in ${PROVER_TESTS} ${VENDOR_TESTS}; do \
			diem move prove -f $$i; \
	done