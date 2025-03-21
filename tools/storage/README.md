# Libra Storage Tool Guide

## Quick Start
The github repo: https://github.com/0LNetworkCommunity/epoch-archive-mainnet contains periodic snapshots of the chain.

To download and restore a snapshot:

```bash
# Download a restore bundle for epoch 339
cargo run -- download-restore-bundle \
  --epoch 339 \
  --destination $HOME/.libra/restore

# Restore the downloaded bundle
cargo run -- epoch-restore \
  --bundle-path $HOME/.libra/restore/epoch_339_restore_bundle \
  --destination-db $HOME/.libra/db_339
```

At the end of a successful run you should see:
```
SUCCESS: restored to epoch: 339, version: 117583050
```

## Restore Bundle Components

A restore bundle contains three components from the GitHub repository's `snapshots/` directory:

### 1. Epoch Ending Files
- Format: `epoch_ending_{epoch}-.*`
- Purpose: Block metadata and proofs
- Example: `epoch_ending_339-.a5b2/`

### 2. State Files
- Format: `state_epoch_{epoch}_ver_{version}.*`
- Purpose: State tree snapshots
- Example: `state_epoch_339_ver_117583050.05f3/`

### 3. Transaction Files
- Format: `transaction_{version}-.*`
- Purpose: Transaction history
- Example: `transaction_117500000-.f674/`

## Download Process

The tool locates three folders for a given epoch:

### Epoch Ending Selection
- Finds exact match for target epoch
- Example: For epoch 339, finds `epoch_ending_339-.a5b2`

### State Epoch Selection
- Matches target epoch and gets associated version
- Example: `state_epoch_339_ver_117583050.05f3`

### Transaction Selection
- Gets version number from state epoch folder name
- Finds transaction folder with highest version below target
- Example: For version 117583050, selects `transaction_117500000`

> **Note**: Some transaction folders may be missing. Last verified working with epoch 339.

## Restore Process

### 1. Decompression Phase
- Scans for `.gz` files in bundle directory
- Decompresses all found files
- Updates manifest files to remove `.gz` extensions

### 2. Bundle Loading Phase
- Loads and validates epoch manifest
- Sets version from waypoint
- Locates state snapshot manifest
- Finds matching transaction manifest

### 3. Database Restoration Phase
- Restores epoch ending data
- Applies state snapshot
- Replays transactions

## Troubleshooting

### Manifest Path Issues
- **Problem**: Manifests may still reference `.gz` files after decompression
- **Solution**: Automatic manifest updating
- **Fallback**: Manual manifest path correction

### Missing Transaction Folders
- **Problem**: Repository may lack needed transaction folders
- **Solution**: Try different epoch or locate correct folder manually

### Version Mismatches
- **Problem**: State/transaction version mismatch
- **Solution**: Verify folder compatibility

### Epoch-Specific Issues
Tool verified with epoch 339. Other epochs may have:
- Different naming patterns
- Missing components
- Varying compression formats

### Verification Steps
1. Check all required folders downloaded
2. Verify manifest file paths
3. Confirm version alignment

## Future Improvements
- [ ] Better missing transaction folder handling
- [ ] Enhanced manifest path updating
- [ ] Support for varying epoch folder patterns
- [ ] Clearer error messaging

## Advanced Usage: Manual Restore

For advanced users who need to restore individual components, you can restore each backup type separately.
This requires understanding the relationships between epoch, state version, and transaction version.

### Environment Setup
```bash
# Set paths for your restore
BACKUP_DIR="path/to/restore/bundle"
DB_PATH="$HOME/.libra/data/db"
TARGET_VERSION=38180075

# Manifest paths in the backup directory
EPOCH_MANIFEST="$BACKUP_DIR/epoch_ending_116-.be9b/epoch_ending.manifest"
STATE_MANIFEST="$BACKUP_DIR/state_epoch_116_ver_38180075.05af/state.manifest"
TX_MANIFEST="$BACKUP_DIR/transaction_38100001-.541f/transaction.manifest"
```

### Individual Restore Commands

#### 1. Restore Epoch Ending
```bash
libra ops storage db restore oneoff epoch-ending \
  --epoch-ending-manifest "$EPOCH_MANIFEST" \
  --target-db-dir "$DB_PATH" \
  --local-fs-dir "$BACKUP_DIR" \
  --target-version "$TARGET_VERSION"
```

#### 2. Restore State Snapshot
```bash
libra ops storage db restore oneoff state-snapshot \
  --state-manifest "$STATE_MANIFEST" \
  --target-db-dir "$DB_PATH" \
  --local-fs-dir "$BACKUP_DIR" \
  --restore-mode default \
  --state-into-version "$TARGET_VERSION" \
  --target-version "$TARGET_VERSION"
```

#### 3. Restore Transactions
```bash
libra ops storage db restore oneoff transaction \
  --transaction-manifest "$TX_MANIFEST" \
  --target-db-dir "$DB_PATH" \
  --local-fs-dir "$BACKUP_DIR" \
  --target-version "$TARGET_VERSION"
```

### Important Notes

- The restore order matters: epoch ending → state snapshot → transactions
- All components must be compatible (epoch and version numbers align)
- The `--target-version` should match across all commands
- The `--local-fs-dir` must contain all referenced files
- Ensure DB_PATH has proper permissions

> **Warning**: Manual restore requires careful attention to version numbers and file paths.
> Consider using the automated `epoch-restore` command for most cases.
