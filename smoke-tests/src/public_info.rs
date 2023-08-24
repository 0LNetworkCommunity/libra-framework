trait LibraPublicInfo {
    async fn ol_create_user_account(&self, address: AccountAddress) -> Result<(), Error>;
}

impl LibraPublicInfo for DiemPublicInfo {
    async fn ol_create_user_account(&self, address: AccountAddress) -> Result<(), Error> {
        let preimage = AuthenticationKeyPreimage::ed25519(pubkey);
        let auth_key = AuthenticationKey::from_preimage(&preimage);
        let create_account_txn =
            self.root_account
                .sign_with_transaction_builder(self.transaction_factory().payload(
                    aptos_stdlib::aptos_account_create_account(auth_key.derived_address()),
                ));
        self.rest_client
            .submit_and_wait(&create_account_txn)
            .await?;
        Ok(())
    }
}
