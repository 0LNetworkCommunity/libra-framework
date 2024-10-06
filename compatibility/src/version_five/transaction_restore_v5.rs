#[allow(dead_code)]
struct LoadedChunk {
    pub manifest: TransactionChunk,
    pub txns: Vec<Transaction>,
    pub txn_infos: Vec<TransactionInfo>,
    pub event_vecs: Vec<Vec<ContractEvent>>,
    pub range_proof: TransactionAccumulatorRangeProof,
    pub ledger_info: LedgerInfoWithSignatures,
}

impl LoadedChunk {
    async fn load(
        manifest: TransactionChunk,
        storage: &Arc<dyn BackupStorage>,
        epoch_history: Option<&Arc<EpochHistory>>,
    ) -> Result<Self> {
        let mut file = BufReader::new(storage.open_for_read(&manifest.transactions).await?);
        let mut txns = Vec::new();
        let mut txn_infos = Vec::new();
        let mut event_vecs = Vec::new();

        while let Some(record_bytes) = file.read_record_bytes().await? {
            let (txn, txn_info, events) = bcs::from_bytes(&record_bytes)?;
            txns.push(txn);
            txn_infos.push(txn_info);
            event_vecs.push(events);
        }

        ensure!(
            manifest.first_version + (txns.len() as Version) == manifest.last_version + 1,
            "Number of items in chunks doesn't match that in manifest. first_version: {}, last_version: {}, items in chunk: {}",
            manifest.first_version,
            manifest.last_version,
            txns.len(),
        );

        let (range_proof, ledger_info) = storage
            .load_bcs_file::<(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)>(
                &manifest.proof,
            )
            .await?;
        if let Some(epoch_history) = epoch_history {
            epoch_history.verify_ledger_info(&ledger_info)?;
        }

        // make a `TransactionListWithProof` to reuse its verification code.
        let txn_list_with_proof = TransactionListWithProof::new(
            txns,
            Some(event_vecs),
            Some(manifest.first_version),
            TransactionInfoListWithProof::new(range_proof, txn_infos),
        );
        txn_list_with_proof.verify(ledger_info.ledger_info(), Some(manifest.first_version))?;
        // and disassemble it to get things back.
        let txns = txn_list_with_proof.transactions;
        let range_proof = txn_list_with_proof
            .proof
            .ledger_info_to_transaction_infos_proof;
        let txn_infos = txn_list_with_proof.proof.transaction_infos;
        let event_vecs = txn_list_with_proof.events.expect("unknown to be Some.");

        Ok(Self {
            manifest,
            txns,
            txn_infos,
            event_vecs,
            range_proof,
            ledger_info,
        })
    }
}
