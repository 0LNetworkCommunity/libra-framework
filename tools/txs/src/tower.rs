use super::submit_transaction::Sender;
use anyhow::bail;
use diem_sdk::rest_client::diem_api_types::TransactionOnChainData;
use libra_cached_packages::libra_framework_sdk_builder::EntryFunctionCall::TowerStateMinerstateCommit;
use libra_types::legacy_types::block::VDFProof;

impl Sender {
    // TODO: should return a UserTransaction as does transfer.rs
    pub async fn commit_proof(
        &mut self,
        proof: VDFProof,
    ) -> anyhow::Result<TransactionOnChainData> {
        if proof.difficulty.is_none() || proof.security.is_none() {
            bail!("no difficulty or security parameter found");
        };

        let payload = TowerStateMinerstateCommit {
            challenge: proof.preimage,
            solution: proof.proof,
            difficulty: proof.difficulty.expect("no difficulty parameter"),
            security: proof.security.expect("no security parameter"),
        }
        .encode();

        self.sign_submit_wait(payload).await
    }
}
