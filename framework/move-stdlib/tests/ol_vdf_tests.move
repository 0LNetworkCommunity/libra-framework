// todo v7: this was working, it is not now
//
// #[test_only]
// module ol_framework::ol_vdf_tests {
//     use ol_framework::vdf;
//     // use std::ol_debug; // todo
//     use ol_framework::test_fixtures;

//     #[test]
//     fun extract_address() {
//         let challenge = test_fixtures::eve_0_easy_chal();
//         // Parse key and check
//         let (eve_addr, _auth_key) = vdf::extract_address_from_challenge(&challenge);
//         assert!(eve_addr == @0x3DC18D1CF61FAAC6AC70E3A63F062E4B, 7357401001);
//     }
// }