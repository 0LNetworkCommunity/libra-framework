# prover tests use a lot of memory so we
# run each module sequentially

# Prover Tests are WIP
# These are the prover tests that have been written
# and are known to pass
PROVER_TESTS = demo slow sacred

VENDOR_TESTS = guid

# Formal verification of each framework using the Move prover
prove:
	@cd move-stdlib && \
	echo "Testing move-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -I {} sh -c 'echo "Testing file: {}"; libra move prove -f {} || exit 1'

	@cd vendor-stdlib && \
	echo "Testing vendor-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -I {} sh -c 'echo "Testing file: {}"; libra move prove -f {} || exit 1'

	@cd libra-framework && \
	for i in ${PROVER_TESTS} ${VENDOR_TESTS}; do \
		libra move prove -f $$i || exit 1; \
	done
