/// Define the GovernanceProposal that will be used as part of on-chain governance by DiemGovernance.
///
/// This is separate from the DiemGovernance module to avoid circular dependency between DiemGovernance and Stake.
module diem_framework::governance_proposal {
    friend diem_framework::diem_governance;

    struct GovernanceProposal has store, drop {}

    /// Create and return a GovernanceProposal resource. Can only be called by DiemGovernance
    public(friend) fun create_proposal(): GovernanceProposal {
        GovernanceProposal {}
    }

    /// Useful for DiemGovernance to create an empty proposal as proof.
    public(friend) fun create_empty_proposal(): GovernanceProposal {
        create_proposal()
    }

    #[test_only]
    public fun create_test_proposal(): GovernanceProposal {
        create_empty_proposal()
    }
}
