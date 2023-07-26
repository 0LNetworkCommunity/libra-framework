# Publishing Smart Contracts
The `txs` tool is responsible for sending all kinds of transctions including the Package Publish transaction.

See an example of a Move package in `./tests/fixtures/test_publish`, which we use for internal "smoke tests".
## Compiling
Note that `txs publish` will compile the package before submitting.
You should test and compile the code before runnin, but it is not necessary for publishing.

## Named addresses
Your module needs to be deployed at an address. You can hard-code the address in the Move.toml file. Or you can have the address variable (e.g. `this_address`), defined at publishing time. In either case the address MUST MATCH THE ADDRESS OF THE SIGNER that is publishing.


### hard-coding address

Move.toml
```
[package]
name = 'test_publish'
version = '1.0.0'

[addresses]
this_address='0000000000000000000000000000000069a385e1744e33fbb24a42ecbd1603e3'

```

contract.move
```
module this_address::message {
    .... <code goes here>
```

### Set name at publishing

You can use the `--named-addresses` cli argument to pass pairs of strings which correspond to the alias (key), and address (value).
```
txs publish --package-dir ./path/to/Move/code --named-addresses "this_address" "0x1234"
```