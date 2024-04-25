# Key Rotation
> CAUTION: Please read carefully and ensure you understand these instructions. Rotating the wrong key could lock you out of your account and make funds permanently inaccessible.

## Prepare a New Authentication Key

### Create a New Address
Generate a fresh address to serve as the new Authentication Key. This command may prompt you to overwrite an existing private key file, which might not be suitable depending on your setup. Use the `-o` flag to specify a different output directory.

```bash
libra wallet keygen
```

### Recover Private Key
To recover a private key using a mnemonic, use:

```bash
libra wallet keygen --mnemonic <MNEMONIC> --output-dir <OUTPUT_DIR>
```

Your private key will be stored in a file called `private_keys.yaml` in the directory you specified above. Specifically called `account_private_key`. The private key corresponds with the `account_address` above it.



## Rotate Keys on Your Wallet

### Rotate Authentication Key
Rotate the authentication key using the new private key for the address associated with the mnemonic you provide upon prompt:

```bash
libra txs user rotate-key --new-private-key <NEW_PRIVATE_KEY>
```

## Delegate the Ability to Rotate Authentication Key

### Delegate Rotation Capability
Grant another user the capability to change the Authentication Key for a specified address. You will be prompted to enter the mnemonic for the address whose authentication key will be changed:

```bash
libra txs user rotation-capability --delegate-address <DELEGATE_ADDRESS>
```

The specified delegate address can now rotate authentication keys on the address for which the mnemonic was provided.

## Rotate Authentication Keys Using a Delegated Address

### Use Delegated Authority to Rotate Key
Enables a delegated user to rotate the Authentication Key for a specified wallet address:

```bash
libra txs user rotate-key --new-private-key <NEW_PRIVATE_KEY> --account-address <ACCOUNT_ADDRESS>