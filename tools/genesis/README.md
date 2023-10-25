## Genesis


### Migration Math

#### `--target-supply <float>` Change in denomination (split)

To adjust the count of coins, and how they get split, the genesis command offers
one input: `--target-supply`.

If for example the supply will scale (pro-rata) from 2,000,000 to 100,000,000,
then the genesis supply calculator (`set_ratios_from_settings()`) will create a split of 50 redenominated coins
for 1 coin. Coins do not change name, no changes in ownership, and all policies
remain the same.

#### `--years-escrow <integer>` Provision an infrastructure escrow

If the validators will set aside rewards for future validators this is done
with: `--years-escrow`. If for example this argument is used, the supply
calculator (`set_ratios_from_settings()`) will take the *remainder* of the
`--target-future-uses` which isn't already in a community-wallet.

#### `--target-future-uses <float>` Community wallet and infra escrow target percentage of network supply
This parameter affects the expected funds available for infra-escrow pledges,
and the daily epoch reward budget.

Note: this could have been implemented as a number targetting the infra-escrow percentage
(e.g. `target-infra-escrow`). However the user-experience of the validator at
genesis is more difficult in this case since the community wallet numbers are
not easily calculated in advance (it would require multiple steps with this tool).

We calculate the infra-escrow budget by working backwards from one input the
community would already have: a target for "future users" (versus end-user
accounts).
If for example the community expected that the combined infra-escrow and
community wallets will represent 70% of the network as a target, we deduce that
infra-escrow will be the remainder of `((0.70 * total supply) - (sum of community
wallets))`.

A lower amount of `--target-future-uses` means a lower amount available to
infrastructure escrow pledges to use over the time `--years-escrow`. i.e. if
target future uses is 55%  (`--target-future-uses 0.55`) and the community
wallet balance is 45%, then there is
10% of supply to be spread over 7 years (`--years-escrow 7`).

Note also we derive the baseline
daily validator rewards from those two parameters. In the example above the
daily reward baseline equals `(10% * Total
Supply) / 7 (years) * 100 validators (baseline) * 365 (days)`

Troubleshooting. If the target percent % is below the proportion of the sum of community
accounts the program will exit with error.


#### `--map_dd_to_slow <list of space delimited addresses>`. Adjusting the future-uses calculator

Ususally in test-cases, there may be cases that the future-uses calculator gets
skewed because certain accounts are not in the expected state (a community
wallet wasn't expected to exist at that point).

