# Wallet

## Legacy Diem Wallets

## HKDF wallets

## Component Overview

`core`

The core directory contains modules for managing key derivation and 
wallet functionalities. It includes implementations for key derivation (key_factory.rs), key generation 
for different roles in the 0L blockchain (legacy_scheme.rs),
mnemonic seed generation and validation (mnemonic.rs), and 
basic wallet management (wallet_library.rs).

`legacy`

The legacy directory focuses on handling components and functionalities related to 
older versions of the blockchain. It includes modules that 
ensure compatibility and proper handling of legacy data 
structures, such as the LegacyAddress struct for representing 
and managing account addresses used in earlier versions of the 
0L blockchain.

`src`

The src directory contains the main source code for the 
wallet, including modules for key management, cryptographic operations, wallet CLI, validator 
configuration, utility functions, and the main application 
entry point. It organizes and facilitates all core 
functionalities necessary for the wallet's operation and 
management.