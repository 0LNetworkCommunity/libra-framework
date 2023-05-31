# txs

0L txs cli tool for sending transactions

This initial version of txs is a combination of clap cli tool lib. and Diem sdk/transfer-coin example modified to work with local tesnet and using 0L keys. 
See txs_args.rs, transfer_coin.rs  

Sources:  
https://diem.dev/tutorials/your-first-transaction/  
App https://github.com/aptos-labs/aptos-core/tree/main/sdk/examples  
Lib https://github.com/aptos-labs/aptos-core/tree/main/sdk/src   

## Example Usage - demo-tx Cmd

```
1. Start local testnet, make sure Diem node and faucet are running

cargo run -p diem -- node run-local-testnet --with-faucet --faucet-port 8081 --force-restart --assume-yes
...
Diem is running, press ctrl-c to exit
Faucet is running. Faucet endpoint: 0.0.0.0:8081


2. Run txs demo cmd

cargo r -- --demo-tx
or 
target/debug/txs --demo-tx

running demo tx ...
--- tc::main: pri: [c4, 3f, 57, 99, 46, 44, eb, da, 1e, ab, fe, bf, 84, de, f7, 3f, bd, 1d, 3c, e4, 42, a9, d2, b2, f4, cb, 9f, 4d, a7, b9, 90, 8c]
--- tc::main: pri: [196, 63, 87, 153, 70, 68, 235, 218, 30, 171, 254, 191, 132, 222, 247, 63, 189, 29, 60, 228, 66, 169, 210, 178, 244, 203, 159, 77, 167, 185, 144, 140]
--- tc::main: pub: Ed25519PublicKey(ef00c7b6f6246543445a847a6d136d293c107b05044f7fc105a063c93c50d7a0)
--- tc::main: aut: AuthenticationKey([fd, a0, 39, 92, f6, 66, 87, 5d, df, 85, 41, 93, fc, cd, 3e, 62, ea, 11, 1d, 6, 60, 29, 49, d, d3, 7c, 89, 1e, d9, c3, f8, 80])
--- tc::main: add: fda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880

=== Addresses ===
Alice: 0xfda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880
Bob: 0x2c929da2b537c51c5db4b5b71826757e7db21780413d362d350026f75f6f47ed

=== Initial Balances ===
Alice: 100000000
Bob: 0
--- sign_transaction(): addr fda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880
--- sign_transaction(): pub Ed25519PublicKey(ef00c7b6f6246543445a847a6d136d293c107b05044f7fc105a063c93c50d7a0)
--- sign_transaction(): pri [196, 63, 87, 153, 70, 68, 235, 218, 30, 171, 254, 191, 132, 222, 247, 63, 189, 29, 60, 228, 66, 169, 210, 178, 244, 203, 159, 77, 167, 185, 144, 140]

=== Intermediate Balances ===
Alice: 99998400
Bob: 1000
--- sign_transaction(): addr fda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880
--- sign_transaction(): pub Ed25519PublicKey(ef00c7b6f6246543445a847a6d136d293c107b05044f7fc105a063c93c50d7a0)
--- sign_transaction(): pri [196, 63, 87, 153, 70, 68, 235, 218, 30, 171, 254, 191, 132, 222, 247, 63, 189, 29, 60, 228, 66, 169, 210, 178, 244, 203, 159, 77, 167, 185, 144, 140]
...
```

## Info

This SDK provides all the necessary components for building on top of the Diem Blockchain. Some of the important modules are:

* `client` - Includes a [REST client](https://diem.dev/nodes/diem-api-spec#/) implementation
* `crypto` - Types used for signing and verifying
* `transaction_builder` - Includes helpers for constructing transactions
* `types` - Includes types for Diem on-chain data structures
