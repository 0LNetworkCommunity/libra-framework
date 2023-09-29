/// Provides a common place for exporting `create_signer` across the Diem Framework.
///
/// To use create_signer, add the module below, such that:
/// `friend diem_framework::friend_wants_create_signer`
/// where `friend_wants_create_signer` is the module that needs `create_signer`.
///
/// Note, that this is only available within the Diem Framework.
///
/// This exists to make auditing straight forward and to limit the need to depend
/// on account to have access to this.
module diem_framework::create_signer {
    friend diem_framework::account;
    #[test_only]
    friend diem_framework::diem_account;
    friend diem_framework::genesis;
    friend diem_framework::multisig_account;
    friend diem_framework::object;

    //////// 0L ////////
    friend ol_framework::fee_maker;

    public(friend) native fun create_signer(addr: address): signer;
}
