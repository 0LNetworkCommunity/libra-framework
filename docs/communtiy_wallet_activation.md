# Community Wallet Activation

Community Wallet is a qualification an account can receive if it is composed
of certain properties. Those properties are:

- Donor Voice, makes payments with a policy where the donors have observability over the transactions.
- Multisig, accounts can only be manipulated with a multisig policy.
- Ancestry limits, multisig authorities must be a minimum of 3 addresses which are not related by Ancestry.
- Caged, the original authentication key is rotated; similar to resource accounts, but further restricted since they cannot sign arbitrary scripts.

## Steps

There are three steps in creating a community wallet account:

1. **Make it a Donor Voice account, and propose the offer to the authorities.**

   This step is atomic. If the proposed authorities do not qualify, the account will not be initialized with Donor Voice features, and the authority offer will not be made. This ensures the community wallet has the expected authorities before proceeding. _Note: The authority offer expires in 7 epochs after this step is executed._

2. **The authorities claim the offer over the account.**

   This step ensures that the proposed authorities acknowledge and accept their roles in managing the community wallet, reinforcing the multisig setup security.

3. **Finalize the account, making it a multisig with the authorities claimed above.**

   This step is irreversible. The current mnemonic will no longer work going forward. The only governance possible is that of the multisig, and any features added (Donor Voice).

## Using TXS Tool

To exemplify the usage of the TXS Tool, we will consider Alice (0x1000a), Bob (0x1000b), and Carol (0x1000c) as the ones invited to be the multisig authorities on Dave's (0x1000d) account, which will become the community wallet.

### Step #1:

Dave simply provides the list of addresses and the signature threshold. The transaction will be rejected if the proposed accounts do not qualify in the simple sybil resistance ancestry check.

_Note: this step does not commit the accounts as the authorities, it proposes the authority offer to the address list._

```
libra txs community gov-init -a 0x1000a -a 0x1000b -a 0x1000c -n 2
```

### Step #2:

Each address owner invited to be an authority in the community wallet must claim the offer by executing this command.

_Note: this will not make the address a multisig authority yet. After enough addresses claim the offer, the invited authorities need to wait for the donor's final action._

```
libra txs community gov-claim -a 0x1000d
```

### Step #3:

After enough addresses claim the offer, Dave can finalizes and cages the account by providing the threshold number.

```
libra txs community gov-cage -n 2
```

If the transaction succeeds, Dave's account will finally become a community wallet and a multisig account. Action proposals and votes on the community wallet can then be initiated by the authorities.

_Note: If an invited authority does not claim the offer and the account is finalized, they will no longer be able to claim the offer. To become an authority after finalization, their addition must be voted on by the existing authorities of the new community wallet._

## Intermediate Optional Step: Updating the Authority Offer

After initializing the community wallet with `gov-init` and before finalizing it with `gov-cage`, the owner has the option to update the authority offer. This allows the owner to change the initial list of proposed authorities if necessary.

This step also performs the same authority verification as `gov-init`.

To update the authority offer, the owner can use the following command:

```
libra txs community gov-offer -a 0x1000b -a 0x1000c -a 0x1000d -a 0x1000e -n 3
```

In this example, Dave is opting to extend the offer to include Eve (0x1000e) as well.

If any authority already claimed the offer and remains on the updated list, they do not need to claim again. However, if any authority is removed from the list, even if they had previously claimed, they will not be part of the community wallet authorities when the account is caged by the donor.

Additionally, this command can be used by the account owner to renew the offer's deadline if it has expired and the authorities have not yet made their claims.
