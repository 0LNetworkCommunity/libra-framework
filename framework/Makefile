# Prover Tests are WIP
# These are the prover tests that have been written
# and are known to pass
prove:

	@cd move-stdlib && \
	echo "Testing move-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -0 -I {} sh -c 'echo "Testing file: {}"; if ! diem move prove -f {} | grep -q "\"Result\": \"Success\""; then echo "Error in file: {}"; exit 1; fi'

	@cd vendor-stdlib && \
	echo "Testing vendor-stdlib" && \
	find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
	xargs -0 -I {} sh -c 'echo "Testing file: {}"; if ! diem move prove -f {} | grep -q "\"Result\": \"Success\""; then echo "Error in file: {}"; exit 1; fi'

	@cd libra-framework && \
	echo "Testing libra-framework" && \
	find . -type f -name "*.move" ! -name "*.spec.move" -print0 | \
	awk -v RS='\0' -v ORS='\0' '{sub(/^\.\//,""); print}' | \
	sort -uz | \
	xargs -0 -I {} sh -c 'echo "Testing file: {}"; if ! diem move prove -f {} | grep -q "\"Result\": \"Success\""; then echo "Error in file: {}"; exit 1; fi'