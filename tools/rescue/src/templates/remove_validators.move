script {
    use diem_framework::validator_set;
    fun main(diem_root: signer) {
        {{#each addresses}}
        validator_set::remove_validator(&diem_root, @0x{{this}});
        {{/each}}
    }
}
