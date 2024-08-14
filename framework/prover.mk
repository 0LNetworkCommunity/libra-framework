# Prover Tests are WIP
# These are the prover tests that have been written
# and are known to pass
PROVER_TESTS = demo ol_account slow sacred

VENDOR_TESTS = chain_id guid

# Formal verification of each framework using the Move prover
prove:
	@cd move-stdlib && \
	echo "Testing move-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -I {} sh -c 'echo "Testing file: {}"; libra move prove -f {} || echo "Error in file: {}"'

	@cd vendor-stdlib && \
	echo "Testing vendor-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -I {} sh -c 'echo "Testing file: {}"; libra move prove -f {} || echo "Error in file: {}"'

	@cd libra-framework && \
	for i in ${PROVER_TESTS} ${VENDOR_TESTS}; do \
			libra move prove -f $$i; \
	done

#TODO: automate libra-framework verification once we have identified and fixed specifications
# @cd libra-framework && \
# echo "Testing libra-framework" && \
# find . -type f -name "*.move" ! -name "*.spec.move" -print0 | \
# awk -v RS='\0' -v ORS='\0' '{sub(/^\.\//,""); print}' | \
# sort -uz | \
# xargs -0 -I {} sh -c 'echo "Testing file: {}"; diem move prove -f {} || echo "Error in file: {}"'
