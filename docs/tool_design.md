
## About Libra Tool Design
The tools are intended to be minimalist, yet modular. Upstream vendors have sophisticated and complex tooling. This is usually unwieldy for the profile of typical 0L users.

### The Customer
If you have a typical end-user use case, Carpe will likely be all you need.
These tools are for users which are engaged in more admin level operations on the network: configuring and querying contracts.

For that user the cli tools here will like have sufficient features: query, transact, run node.

Similarly if you are a Move dev, similarly the most common features are covered: testing, verifying, compiling, deploying.

### Bring your own tool
If you have needs that aren't met with these tools, all of them are also exported as libraries. Meaning: they are architected so that they are easy to extend.

#### Start a new minimal Rust crate
With a simple Rust project, that uses Clap as a CLI scaffold, you can import all of the CLI types, whole or in part. This means you can extend the existing methods (by creating a `trait` extension in your own tool).

Additionally the most relevant vendor SDK types are re-exported by `libra-types`. So you should be able to take advantage of much of the Move resource parsing (e.g converting account addresses from API calls to structs);

