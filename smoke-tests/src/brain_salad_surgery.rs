//     _________
//     /         \
//   |  @@@@@@@  |
//   | @@     @@ |
//   | @@     @@ |
//   |  @@@@@@@  |
//     \_______/
//   /           \
// |  ()     ()  |
// |     ---     |
// |    \___/    |
//   \_________/

// Note: swarm has several limitations
// Most swarm and LocalNode properties are not mutable
// after init.
// For example: it's not possible to point a swarm node to a new config file
// in a different directory. It is always self.directory.join("node.yaml")
// Similarly it's not possible to change the directory itself.
// So these tests are a bit of a hack.
// we replace some files in the original directories that
// swarm created, using same file names. The function used is called
// brain_salad_surgery().
use crate::helpers::creates_random_val_account;
use diem_forge::LocalSwarm;
use fs_extra::dir;
use tokio::fs;

/// swaps out the
/// preserving the external structure.
pub async fn brain_salad_surgery(swarm: &LocalSwarm) -> anyhow::Result<()> {
    // Write the modified config back to the original location
    let swarm_dir = swarm.dir();
    let backup_path = swarm_dir.join("bak");

    for (i, n) in swarm.validators().enumerate() {
        let node_data_path = n.config_path();
        let node_data_path = node_data_path.parent().unwrap();

        if !backup_path.exists() {
            fs::create_dir_all(&backup_path).await?;
        }
        fs_extra::move_items(&[node_data_path], &backup_path, &dir::CopyOptions::new())?;

        if !node_data_path.exists() {
            fs::create_dir_all(&node_data_path).await?;
        }
        let cfg = n.config();
        let net = cfg.validator_network.iter().next().unwrap();
        let port = net.listen_address.find_port().expect("to find port");
        creates_random_val_account(node_data_path, port).await?;

        // Swarm uses a fixed node.yaml for the config file,
        // while we use role-based names.
        // we'll deprecate the one we would normally use.
        fs::rename(
            node_data_path.join("validator.yaml"),
            node_data_path.join("validator.depr"),
        )
        .await?;

        fs::rename(
            node_data_path.join("public-keys.yaml"),
            node_data_path.join("public-identity.yaml"),
        )
        .await?;
        fs::rename(
            node_data_path.join("private-keys.yaml"),
            node_data_path.join("private-identity.yaml"),
        )
        .await?;

        fs::rename(
            node_data_path.join("validator-full-node-identity.yaml"),
            node_data_path.join("vfn-identity.yaml"),
        )
        .await?;
        // now copy back the node.yaml, it has randomly generated addresses
        // and is generally in a format the swarm tool will understand
        fs::copy(
            backup_path.join(format!("{i}/node.yaml")),
            node_data_path.join("node.yaml"),
        )
        .await?;

        // NOTE: devs if you need the swarm genesis.blob in the future uncomment
        // fs::copy(
        //     backup_path.join(&format!("{i}/genesis.blob")),
        //     node_data_path.join("genesis.blob"),
        // )
        // .await?;

        // and copy the db which we will modify
        fs_extra::dir::copy(
            backup_path.join(format!("{i}/db")),
            node_data_path,
            &dir::CopyOptions::new(),
        )?;
    }

    // We've got a ballad
    // About a salad brain
    // With assurgence
    // In a dirty bit again

    // [Verse 2]
    // Brain salad certainty
    // It will work for you, it works for me
    // Brain rot perversity
    // Brain salad surgery
    Ok(())
}

#[tokio::test]
// can restart after brain salad surgery
// but will not progress
async fn test_brain_salad() -> anyhow::Result<()> {
    let mut s = crate::libra_smoke::LibraSmoke::test_setup_start_then_pause(2).await?;

    brain_salad_surgery(&s.swarm).await?;

    for node in s.swarm.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = false;
        crate::helpers::update_node_config_restart(node, &mut node_config)?;
    }
    Ok(())
}
