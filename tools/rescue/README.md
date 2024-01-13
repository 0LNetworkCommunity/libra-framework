# Rescue Missions

## TL;DR
The same tools used to manufacture on-chain governance proposals
(`libra-framework`) are used to create governance scripts.
These scripts can be executed offline with db-bootstrapper via the `rescue-cli`
tool here.

# Framework upgrades
For now you need to craft a frankenstein TX by hand. A better tool could be
built, but a rescue operation is a rare occurrance and should be led by people
familiar with the tools.

Creating the bytes for publishing a new/updated Move module is done by the
libra-framework tool;
simply `cargo run upgrade`. You can see the instructions there to
create new Move module compile artifacts.

See an example in the test fixtures here.
This is aspecific case of of an upgrade while a network is halted. We have
copied the bits from upgrade examples in
`framework/src/upgrade_fixtures/fixtures/upgrade-multi-lib/3-libra-framework/sources/3-libra-framework.move`
this file will include a test module: all_your_base.move

# HACK THE BLACK MAGIC
When trying to boostrap a db, and get a valid state transition, we need the transaction to emit a "reconfiguration event":
a reconfiguration event must:
- happen with timestamp not equal to previous reconfiguration
- have a "touch" to validator set, whatever that means