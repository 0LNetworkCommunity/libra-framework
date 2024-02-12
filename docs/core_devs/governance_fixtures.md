# Governance fixtures

To test governance scripts and framework upgrade we place
tests in `tools/txs/tests` and fixture scripts in `tools/txs/tests/fixtures`.

To generate these fixtures we use the `libra-framework cli`.

##  generate a noop governance script

1. A template Move script package must exist. The CLI can create a template.

```
cd libra-framework
cargo r governance --script-dir ../tools/txs/tests/fixtures --framework-local-dir ./libra-framework --only-make-template
```

2. Don't make changes to this file. It should compile as-is. Run the cli tool again.
```
cd libra-framework
cargo r governance --script-dir ../tools/txs/tests/fixtures/governance_script_template --framework-local-dir ./libra-framework
```

3. check the files are as expected
You should have
```
../tools/txs/tests/fixtures/governance_script_template/
|
+ - /sources/
|   |
|   +-- governance_script_template.move
+ - Move.toml
+ - script_sha3 // hash of the script needed for authorization
+ - script.mv // compiled script which is submitted
```

You're done.