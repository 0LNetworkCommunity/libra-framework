
# Backwards Compatibility

## TL;DR
The encoding of the bytes in libra uses `BCS` which is a de/serialization implementation using `serde`. Libra Version Six and beyond cannot decode transaction logs or state snapshots from V5 without these tools.

# Explain

V6 was a kitchen sink upgrade with a new genesis, since there were upgrades throughout the stack that would have created a discontinuity in blocks anyhow.

The bytes present in prior db, logs, and backups prior to V6 had different memory layouts. For every K-V structure the keys had different hashes, and the values had different encoding layouts.

Also, looking up data in K-V representations of bytes is done with byte encoded access_paths. Since the Move language address format and data structure names have changed, nothing can be found, and you will receive a `remaining input` error. I gift you this koan.

This compatibility library ports the some V5 Rust code so that certain elemental types (StructTag, TypeTag, HashValue, AccountAddress), use the correct layout.

# Principal PITAs

1. Backup Manifests have changed layout. State Snapshot Manifests have changed ever so slightly, they previously did not include "epoch" keys. Reading V5 backup archive manifests would fail with V6+ tooling.

1. `AccountStateBlob` stored bytes are not what they seem: In the State Snapshot backup files, each chunk is represented by a tuple of `(HashedValue, AccountStateBlob)`. For clarity we added a definition of `AccountStateBlobChunkV5`.

1. `HashValue` is evil: The HashValue layout has not changed, but it invokes loup garou vodoo, and the custom deserializer of HashedValue uses a different intermediary representation for the byte layout
```
#[derive(::serde::Deserialize)]
#[serde(rename = "HashValue")]
struct Value<'a>(&'a [u8]);

let value = Value::deserialize(deserializer)?;
Self::from_slice(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
```
1. `AccountAddress` makes everything fail: fixed lengths have changed, from V5 to V6 the addresses doubled in size (32 to 64 bits). No KV lookup will work because the byte-encoded key always has the Core Code Address, (0x1) which changed from being prepended with 16 zeros, to 32 zeros. So all language_storage.rs structs are changed to use `LegacyAddressV5`.

# Tests
The principal tests to run are in `state_snapshot_v5.rs`, where we try to sanity test the encoding and decoding of structs for the v5 language elements.


# References

```
$ >
$ > Loop Garoo
Goin' down to junk anew?
$ > Loop Garoo
Goin' put my hook to you
```
