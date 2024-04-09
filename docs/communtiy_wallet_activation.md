# Community Wallet Activation

There are two steps in creating a community wallet account:
 1) make it a Donor Voice account, and check if the proposed authorites could be
    added.

  This step is atomic, so that if the proposed authorities do not qualify, then
  the account will not be initialized with Donor Voice features. This is a
  safety check to ensure the community wallet has the expected authorities
  before making it irreversibly a multisig.
 2) finalize the account, making it a multisig with the authorities checked
    above.
 This step is irreversible. The current mnemonic will no longer work going
 forward. The only governance possible is that of the multi-sig, and any
 featres added (Donor Voice)

# Using TXS tool

### Step #1:

Simply provide the list of addresses, and the signature threshold. The
transaction will be rejected if the proposed accounts don't qualify in the
simple sybil resistance ancestry check.

Note: this step does not commit the accounts as the authorities, it merely
checks in advance of finalizing.

```

libra txs community gov-init -a 0x1000a -a 0x1000b -a 0x1000b -n 2

```

### Step #2

You should use the same accounts as in Step #1. There's nothign to enforce this,
and you can select a different set of accounts.

```
libra txs community gov-init -a 0x1000a -a 0x1000b -a 0x1000b -n 2 --finalize

```
