# Versions of Libra SDK

## Why do we need these?
Transactions submitted to blockchain are done in bytecode. Decoding that bytecode into a useful object (function name, arguments, timestamp) is hard to do if you do not have the Rust serialization lib which generated the bytecode originally.

The names of functions (and some primitives like Account Address), change between versions.

Suppose you would like to analyze a V5 transaction. Without these files you'll have to guess the transaction type and craft an individual decoder.

# Modifications to originals
Note that these files are code generated, and were not intended to be changed. We have however added serde De/Serialize attributes to the EntryFunctionCall, so that they can be easily be read into memory.
