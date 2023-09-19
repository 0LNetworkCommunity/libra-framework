# Welcome Validators

This assumes you have the `libra` cli installed in your local $PATH.

## Quick start
```
# create account keys
libra wallet keygen

# create the validator config files on your node
libra config validator-init

# a friend will onboard the account if it doesn't yet exist on chain

# send validator info
libra txs validator register

# get vouches from existing validators (just ask)
libra txs validator vouch --vouch-for <YOUR ADDRESS>

# submit a bid to be in the validator set
libra txs validator pof --bid-pct <PERCENT YOU PAY> --expiry <WHEN EXPIRES>

```

# Get Keys
If you don't already have an account, you'll need a mnemonic (seed), to generate all keys.

```
libra wallet keygen
```

# Initialize validator files

Follow the prompts here. Your node needs to have keys generated using a mnemonic from step #1.

```
libra config validator-init
```

# Get the account on chain
Someone needs to create that account onchain first.
Ask someone to deposit a coin to your accout from step #1

```
# friend sends one coin to your account which creates it
libra txs transfer -t <YOUR ACCOUNT> -a 1

```

# Submit configs to chain

```
libra txs validator register

# optionally pass -f to the file where operator.yaml from step #2 above is located
libra txs validator register -f /path/to/foo/operator.yaml

```

# Get Vouches
0L uses very light reputation games to keep the validator set trusted.
Just ask an existing validator for a vouch. It doesn't cost you anything and it needs no stake.

Your friend will:
`libra txs validator vouch --vouch-for <YOUR ADDRESS>`

# Bid to be in the validator set
0L uses Proof-of-Fee for sybil resistance, instead of Proof-of-Stake. You don't need any stake to join, but you just need to be able to bid on how much you are willing to pay to be in the validator set. The cheapest bid proposed by validators will be actually what all validators pay (uniform price auction).

```
libra txs validator pof --bid-pct <PERCENT YOU PAY> --expiry <WHEN EXPIRES>
```
