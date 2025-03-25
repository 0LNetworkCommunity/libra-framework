# Rescue Tool
Apply changes to a reference DB at rest.

# Uses
There are three cases:
1. Twin: replace the validator set from config files
2. Upgrade only: upgrade the framework (maybe the source in reference db is a brick, and can't do the scripts you want).
3. Run script: the framework in reference DB is usable, and we need to execute an admin transaction from a .move source


# Replace validators (Twin testnet)
```
cargo r -- --db-path ~/.libra/data/db \
register-vals \
--operator-yaml ~/.libra/operator.yaml
```
