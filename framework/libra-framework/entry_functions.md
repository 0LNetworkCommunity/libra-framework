# Libra Framework Entry Functions

## About

This document lists entry functions that are publicly callable by end users in the Libra Framework's production code. Test-only functions have been excluded unless they provide important functionality.

The entry functions are organized by their module location and include a brief description of their purpose. For detailed parameter lists and implementation details, please refer to the source code of each module.

Functions marked with 🖥️ are integrated with the command-line interface (CLI), allowing users to call these functions directly through the Libra CLI. For each CLI-integrated function, the corresponding CLI command is listed.

## Core Framework Modules

### account.move

- `rotate_authentication_key(account: &signer, from_scheme: u8, from_public_key_bytes: vector<u8>, to_scheme: u8, to_public_key_bytes: vector<u8>, cap_rotate_key: vector<u8>, cap_update_table: vector<u8>)` 🖥️
  - Rotates the authentication key for an account
  - CLI: `txs user rotate-key`

- `rotate_authentication_key_with_rotation_capability(account_with_cap: &signer, rotate_key_cap: vector<u8>, new_scheme: u8, new_public_key_bytes: vector<u8>, cap_update_table: vector<u8>)` 🖥️
  - Rotates authentication key using rotation capability
  - CLI: `txs user rotate-key` with a delegated account

- `offer_rotation_capability(account: &signer, rotation_capability_sig_bytes: vector<u8>, account_scheme: u8, account_public_key_bytes: vector<u8>, recipient_address: address)` 🖥️
  - Offers rotation capability to another account
  - CLI: `txs user rotation-capability --delegate-address <ADDRESS>`

- `revoke_rotation_capability(account: &signer, to_be_revoked_address: address)` 🖥️
  - Revokes rotation capability from a specific address
  - CLI: `txs user rotation-capability --revoke --delegate-address <ADDRESS>`

- `revoke_any_rotation_capability(account: &signer)`
  - Revokes any rotation capability

### libra_coin.move

- `mint_to_impl(root: &signer, dst_addr: address, amount: u64)`
  - Root account can mint coins to an address
  - Only used for genesis and smoke tests

### slow_wallet.move

- `smoke_test_vm_unlock(smoke_test_core_resource: &signer, user_addr: address, unlocked: u64, transferred: u64)`
  - Sets the unlocked and transferred amounts for a slow wallet
  - Only used for smoke tests

### code.move
- `publish_package_txn(owner: &signer, metadata_serialized: vector<u8>, code: vector<vector<u8>>)` 🖥️
  - Publishes a Move package to the blockchain
  - CLI: `txs publish --package-dir <PATH>`

### diem_governance.move

- `smoke_trigger_epoch(core_resources: &signer)`
  - Triggers a new epoch in test environments
  - Only used for smoke tests

- `ol_create_proposal_v2(proposer: &signer, execution_hash: vector<u8>, metadata_location: vector<u8>, metadata_hash: vector<u8>, is_multi_step_proposal: bool)` 🖥️
  - 0L specific variant for creating proposals
  - CLI: `txs governance propose --proposal-script-dir <DIR> --metadata-url <URL>`

- `ol_vote(voter: &signer, proposal_id: u64, should_pass: bool)` 🖥️
  - 0L specific variant for voting on proposals
  - CLI: `txs governance vote --proposal-id <ID> [--should-fail]`

- `assert_can_resolve(proposal_id: u64)`
  - Verifies if a proposal can be resolved
  - Used for testing

- `trigger_epoch(_sig: &signer)` 🖥️
  - Triggers a new epoch
  - CLI: `txs governance epoch-boundary`

### code.move

- `publish_package_txn(owner: &signer, metadata_serialized: vector<u8>, code: vector<vector<u8>>)` 🖥️
  - Publishes a Move package to the blockchain
  - CLI: `txs publish --package-dir <PATH>`

### multisig_account.move
####  NOTE: not implemented in OL client tools

- `create_with_existing_account(creator: &signer, owners: vector<address>, num_signatures_required: u64)`
  - Creates a multisig account with an existing account

- `create(creator: &signer, owners: vector<address>, num_signatures_required: u64): address`
  - Creates a new multisig account

- `create_with_owners(creator: &signer, owners: vector<address>, num_signatures_required: u64): address`
  - Creates a multisig account with specified owners

- `migrate_with_owners(creator: &signer, additional_owners: vector<address>, num_signatures_required: u64, metadata_keys: vector<vector<u8>>, metadata_values: vector<vector<u8>>)`
  - Migrates a multisig account to new owners

- `create_transaction(owner: &signer, multisig_account: address, payload: vector<u8>)`
  - Creates a new transaction for multisig approval

- `create_transaction_with_hash(owner: &signer, multisig_account: address, metadata_hash: vector<u8>)`
  - Creates a transaction with a specific hash

- `approve_transaction(owner: &signer, multisig_account: address, sequence_number: u64)`
  - Approves a pending multisig transaction

- `reject_transaction(owner: &signer, multisig_account: address, sequence_number: u64)`
  - Rejects a pending multisig transaction

- `vote_transaction(owner: &signer, multisig_account: address, sequence_number: u64, approved: bool)`
  - Votes on a multisig transaction

- `execute_rejected_transaction(owner: &signer, multisig_account: address)`
  - Executes a rejected transaction

- `add_owner(owner: &signer, new_owner: address)`
  - Adds a new owner to a multisig account

- `add_owners(owner: &signer, new_owners: vector<address>)`
  - Adds multiple new owners to a multisig account

- `remove_owner(owner: &signer, owner_to_remove: address)`
  - Removes an owner from a multisig account

- `remove_owners(owner: &signer, owners_to_remove: vector<address>)`
  - Removes multiple owners from a multisig account

- `update_signatures_required(owner: &signer, new_num_signatures_required: u64)`
  - Updates the number of signatures required for transaction approval

- `update_metadata(owner: &signer, keys: vector<vector<u8>>, values: vector<vector<u8>>)`
  - Updates metadata for a multisig account

## OL Sources Modules

### community_wallet_init.move

- `init_community(sig: &signer, initial_authorities: vector<address>, check_threshold: u64)` 🖥️
  - Initializes a community wallet
  - CLI: `txs community gov-init --admins <ADDRESSES> --num-signers <N>`

- `propose_offer(sig: &signer, new_signers: vector<address>, num_signers: u64)` 🖥️
  - Proposes to offer new signers to a community wallet
  - CLI: `txs community gov-offer --admins <ADDRESSES> --num-signers <N>`

- `finalize_and_cage(sig: &signer, num_signers: u64)` 🖥️
  - Finalizes and cages a community wallet
  - CLI: `txs community gov-cage --num-signers <N>`

- `change_signer_community_multisig(sig: &signer, multisig_address: address, new_signer: address, is_add_operation: bool, n_of_m: u64, vote_duration_epochs: u64)` 🖥️
  - Changes signers in a community multisig wallet
  - CLI: `txs community gov-admin --community-wallet <ADDRESS> --admin <ADDRESS> --n <N>`

### donor_voice_txs.move

- `propose_payment_tx(donor: &signer, multisig_address: address, payee: address, value: u64, description: vector<u8>)` 🖥️
  - Proposes a payment transaction
  - CLI: `txs community propose --community-wallet <ADDRESS> --recipient <ADDRESS> --amount <N> --description <DESC>`

- `propose_veto_tx(donor: &signer, multisig_address: address, id: u64)` 🖥️
  - Proposes a veto transaction
  - CLI: `txs community veto --community-wallet <ADDRESS> --proposal-id <ID>`

- `propose_advance_tx(donor: &signer, multisig_address: address, id: u64)`
  - Proposes to advance a transaction without required approvals
  - Used to bypass voting periods in special cases

- `propose_liquidate_tx(donor: &signer, multisig_address: address)`
  - Proposes a liquidation transaction

- `vote_liquidation_tx(donor: &signer, multisig_address: address)` 🖥️
  - Votes on a liquidation transaction
  - CLI: `txs community liquidate --community-wallet <ADDRESS>`

### multi_action.move

- `claim_offer(sig: &signer, multisig_address: address)` 🖥️
  - Claims an offer for a multisig account
  - CLI: `txs community gov-claim --community-wallet <ADDRESS>`

### proof_of_fee.move

- `init_bidding(sender: &signer)`
  - Initializes bidding process

- `pof_update_bid(sender: &signer, bid: u64, epoch_expiry: u64)` 🖥️
  - Updates a Proof of Fee bid
  - CLI: `txs validator pof --bid-pct <PCT> --epoch-expiry <N>`

- `pof_update_bid_net_reward(sender: &signer, net_reward: u64, epoch_expiry: u64)`
  - Updates a Proof of Fee bid with net reward
  - Used for setting the net reward for validators

- `pof_retract_bid(sender: signer)` 🖥️
  - Retracts a Proof of Fee bid
  - CLI: `txs validator pof --retract --epoch-expiry <N>`

### burn.move

- `set_send_community(sender: &signer, community: bool)` 🖥️
  - Sets whether burns should be sent to community
  - CLI: `txs burn-preference --community <BOOL>`

### ol_account.move

- `create_account(root: &signer, auth_key: address)`
  - Creates a new account
  - Used for testing

- `transfer(sender: &signer, to: address, amount: u64)` 🖥️
  - Transfers funds from sender to recipient
  - CLI: `txs transfer --to-account <ADDRESS> --amount <AMOUNT>`

- `set_allow_direct_coin_transfers(account: &signer, allow: bool)`
  - Sets whether direct coin transfers are allowed for an account

### slow_wallet.move

- `user_set_slow(sig: &signer)` 🖥️
  - Allows users to change their account to a slow wallet
  - CLI: `txs user set-slow`

### jail.move

- `unjail_by_voucher(sender: &signer, addr: address)` 🖥️
  - Unjails an account using a voucher
  - CLI: `txs validator jail --unjail-acct <ADDRESS>`

### vouch.move

- `vouch_for(grantor: &signer, friend_account: address)` 🖥️
  - Vouches for a validator candidate
  - CLI: `txs validator vouch --vouch-for <ADDRESS>`

- `revoke(grantor: &signer, friend_account: address)` 🖥️
  - Revokes a vouch for a validator candidate
  - CLI: `txs validator vouch --vouch-for <ADDRESS> --revoke`

- `clean_expired(user_sig: &signer)`
  - Cleans expired vouches for a user

### validator_universe.move

- `register_validator(sig: &signer, consensus_pubkey: vector<u8>, proof_of_possession: vector<u8>, network_addresses: vector<u8>, fullnode_addresses: vector<u8>)` 🖥️
  - Registers a validator in the validator universe
  - CLI: `txs validator register [--operator-file <FILE>]`

### stake.move

- `initialize_validator(owner: &signer, consensus_pubkey: vector<u8>, proof_of_possession: vector<u8>, network_addresses: vector<u8>, fullnode_addresses: vector<u8>)` 🖥️
  - Initializes a validator
  - CLI: `txs validator initialize [--operator-file <FILE>]`

- `update_network_and_fullnode_addresses(owner: &signer, validator: address, new_network_addresses: vector<u8>, new_fullnode_addresses: vector<u8>)` 🖥️
  - Updates a validator's network and fullnode addresses
  - CLI: `txs validator update [--operator-file <FILE>]`
