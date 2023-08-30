use super::client_ext::ClientExt;
use anyhow::Result;
use async_trait::async_trait;
// use crate::type_extensions::client_ext::ClientExt;
use diem_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    rest_client::Client,
    types::{AccountKey, LocalAccount},
};

#[async_trait]
pub trait Ed25519PrivateKeyExt {
    async fn get_account(&self, sequence_number: Option<u64>) -> Result<LocalAccount>;
}

#[async_trait]
impl Ed25519PrivateKeyExt for Ed25519PrivateKey {
    async fn get_account(&self, sequence_number: Option<u64>) -> Result<LocalAccount> {
        let account_key = AccountKey::from_private_key(self.to_owned());
        let account_address = account_key.authentication_key().derived_address();
        let sequence_number = match sequence_number {
            Some(seq) => seq,
            None => {
                let client = Client::default().await?;
                client.get_sequence_number(account_address).await?
            }
        };

        Ok(LocalAccount::new(
            account_address,
            account_key,
            sequence_number,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::type_extensions::ed25519_private_key_ext::Ed25519PrivateKeyExt;
    use diem_sdk::crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt};

    #[tokio::test]
    async fn create_local_account_from_private_key() {
        let private_key = Ed25519PrivateKey::from_encoded_string(
            "c43f57994644ebda1eabfebf84def73fbd1d3ce442a9d2b2f4cb9f4da7b9908c",
        )
        .unwrap();
        let account = private_key.get_account(Some(0)).await.unwrap();
        let expected_public_key =
            "ef00c7b6f6246543445a847a6d136d293c107b05044f7fc105a063c93c50d7a0";
        let expected_auth_key = "fda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880";
        let expected_account_address =
            "0xfda03992f666875ddf854193fccd3e62ea111d066029490dd37c891ed9c3f880";

        assert_eq!(&private_key, account.private_key());
        assert_eq!(expected_public_key, account.public_key().to_string());
        assert_eq!(expected_auth_key, account.authentication_key().to_string());
        assert_eq!(expected_account_address, account.address().to_hex_literal());
    }
}
