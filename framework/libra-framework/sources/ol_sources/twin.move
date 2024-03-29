// Tooling for running offline tests in Twin environment.
// A twin environment is one that recreates a Mainnet DB from snapshot archive
// then runs a DiemDebugger instance on it.
// With the debugger instance we can execute transactions.
// However different than the vendor's DiemDebugger, we will want to run a Swarm
// with the Mainnet state. Which means we need to make some offline Writesets to
// the DB (a genesis bundle similar to Rescue missions).
// These writesets would include:
// - registering new Validators (which are randomly
// generated with Swarm tooling, and wouldn't be found on a Mainnet DB)
// - changing the ChainID, from mainnet
// - changing the Epoch Interval Time.
