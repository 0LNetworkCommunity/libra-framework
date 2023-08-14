
# NOTE: With tests that require compiling the framework, we often run into stack overflows. We then need to set a higher min stack for the runtime.
# Also in CI we want to see where all the failures happened, so `no-fail-fast`
tests:
		RUST_MIN_STACK=104857600 cargo t --no-fail-fast