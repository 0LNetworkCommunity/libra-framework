use std::path::Path;

use diem_genesis::config::OperatorConfiguration;
use diem_types::network_address::NetworkAddress;

#[test]
fn encode_net_addr() -> anyhow::Result<()>{
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/operator.yaml");

    let yaml_str = std::fs::read_to_string(file)?;

    let oc: OperatorConfiguration = serde_yaml::from_str(&yaml_str)?;

    let val_net_protocol = oc
        .validator_host
        .as_network_address(oc.validator_network_public_key)?;
    let enc = bcs::to_bytes(&val_net_protocol)?;

    // dbg!(&hex::encode(&enc));
    let dec: NetworkAddress = bcs::from_bytes(&enc)?;
    dbg!(&dec);

    Ok(())
}
