use diem_forge::LocalNode;
use diem_forge::NodeExt;
use diem_genesis::keys::PublicIdentity;
use libra_config::validator_registration::ValCredentials;
use libra_query::query_view;
use std::fs;

/// Extract the credentials of the random validator
pub async fn extract_swarm_node_credentials(
    marlon_node: &LocalNode,
) -> anyhow::Result<ValCredentials> {
    // get the necessary values from the current db
    let account = marlon_node.config().get_peer_id().unwrap();

    let public_identity_yaml = marlon_node
        .config_path()
        .parent()
        .unwrap()
        .join("public-identity.yaml");
    let public_identity =
        serde_yaml::from_slice::<PublicIdentity>(&fs::read(public_identity_yaml)?)?;
    let proof_of_possession = public_identity
        .consensus_proof_of_possession
        .unwrap()
        .to_bytes()
        .to_vec();
    let consensus_public_key_file = public_identity
        .consensus_public_key
        .clone()
        .unwrap()
        .to_string();

    // query the db for the values
    let query_res = query_view::get_view(
        &marlon_node.rest_client(),
        "0x1::stake::get_validator_config",
        None,
        Some(account.to_string()),
    )
    .await
    .unwrap();

    let network_addresses = query_res[1].as_str().unwrap().strip_prefix("0x").unwrap();
    let fullnode_addresses = query_res[2].as_str().unwrap().strip_prefix("0x").unwrap();
    let consensus_public_key_chain = query_res[0].as_str().unwrap().strip_prefix("0x").unwrap();

    // for checking if both values are the same:
    let consensus_public_key_chain = hex::decode(consensus_public_key_chain).unwrap();
    let consensus_pubkey = hex::decode(consensus_public_key_file).unwrap();
    let network_addresses = hex::decode(network_addresses).unwrap();
    let fullnode_addresses = hex::decode(fullnode_addresses).unwrap();

    assert_eq!(consensus_public_key_chain, consensus_pubkey);
    Ok(ValCredentials {
        account,
        consensus_pubkey,
        proof_of_possession,
        network_addresses,
        fullnode_addresses,
    })
}
