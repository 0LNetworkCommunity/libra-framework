# Key Rotation
> CAUTION: Please read carefully and ensure you understand these instructions. Rotating the wrong key could lock you out of your account and make funds permanently inaccessible.

There are two cases:

1) you are in full control of an account, and would like to
rotate to a new private key (using a new mnemonic).

This is a single step, and you can simply use the current mnemonic (to sign a
transaction), and the new mnemonic (to sign a rotation proof).

2) you are claiming an account from someone else.

This requires two steps where the first current owner will assign one of your
existing on-chain accounts as authorized to rotate keys for the account being
claimed.


## CASE 1 Rotate Keys on Your Wallet

You will be prompted for a mnemonic TWICE. But theses should be DIFFERENT
mnemonics.

The first mnemonic is to create a client to sign and send a transaction to
blockchain.

Then you will be prompted for what NEW mnemonic you would like to be using for
that account going forward.

Additionally, you can expect the CLI tool to ask you to confirm this operation
twice in the process.

```bash
libra txs user rotate-key
```

Note: If you have an advanced case, and would like to submit the private key itself, see below.

## CASE 2: Claim an account

There are two steps involved to claim another account. There are three accounts
- There are two parties Alice (Original Owner) and Bob (New Owner).
- Alice is offering account `0x123` to Bob
(Claimed Account).
- Bob must already have a separate account on chain `0x456` (Delegate Account). The
only reason for this is that Bob needs to do some sensitive signing of keys and
submit it to the chain, and there's no way for Alice or really anyone else to do
this for him. So, this could be a disposable account.

- Bob will also require a New Mnemonic, which is generated offline.

With all that in place, Alice will send a transaction to "delegate" Bob's
account `0x456` with
the power to rotate the keys to `0x123`. Alice's job ends here.
Next bob needs his usual credentials for `0x456`, and also the New Mnemonic he
plans to use for `0x123`.
He submits a transaction (after a bit of processing of the New Mnemonic private
keys), which should successfully rotate the keys to `0x123`

The job of the Delegate account `0x456` is over (the account could even be disposed of).


### Step 1: Alice Delegates Rotation Capability
Grant another user the capability to change the Authentication Key for a specified address. You will be prompted to enter the mnemonic for the address whose authentication key will be changed:

```bash
libra txs user rotation-capability --delegate-address <DELEGATE_ADDRESS>
```

The specified delegate address can now rotate authentication keys on the address for which the mnemonic was provided.

## Step 2: Bob Rotates Authentication Keys Using the Delegated Address

Enables a delegated user to rotate the Authentication Key for a specified wallet address:

```bash
libra txs user rotate-key --claim-address <ACCOUNT_ADDRESS>
```

## Cheat Sheet

### Create a new mnemonic
```
libra wallet keygen
```


### Advanced: Optionally Input the Private Key
To recover a private key using a mnemonic, use:

```bash
libra wallet keygen --mnemonic <MNEMONIC> --output-dir <OUTPUT_DIR>
```

Your private key will be stored in a file called `private_keys.yaml` in the directory you specified above. Specifically called `account_private_key`. The private key corresponds with the `account_address` above it.

Once you have a private key, you can submit the transaction by explicitly
setting the key. In this case the new mnemonic will not be asked for.

```bash
libra txs user rotate-key --new-private-key <NEW_PRIVATE_KEY> --claim-address <ACCOUNT_ADDRESS>
```
