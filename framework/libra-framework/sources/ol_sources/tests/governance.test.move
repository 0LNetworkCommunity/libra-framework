#[test_only]
module ol_framework::test_governance {

  use ol_framework::mock;
  use aptos_framework::aptos_governance;
  use aptos_framework::governance_proposal::GovernanceProposal;
  use aptos_framework::voting;
  use aptos_framework::timestamp;
  // use aptos_std::debug::print;

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  fun gov_threshold_can_resolve_early(root: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 2);
    aptos_governance::initialize_for_test(root);
    let prop_id = aptos_governance::get_next_governance_proposal_id();

    aptos_governance::ol_create_proposal_v2(
      alice,
      x"deadbeef",
      b"hi",
      b"again",
      true,
    );

    let prop_id_next = aptos_governance::get_next_governance_proposal_id();
    assert!(prop_id_next > prop_id, 73570001);

    // alice votes
    aptos_governance::ol_vote(
      alice,
      prop_id,
      true,
    );

    // bob votes
    aptos_governance::ol_vote(
      bob,
      prop_id,
      true,
    );

    let (yes, no) = aptos_governance::get_votes(prop_id);
    assert!(yes == 2, 73570002);
    assert!(no == 0, 73570003);

    // are we above the threshold for early voting?
    let early = voting::is_early_close_possible<GovernanceProposal>(@ol_framework, prop_id);
    assert!(early, 73570004);

    // will it close
    let closed = voting::is_voting_closed<GovernanceProposal>(@ol_framework, prop_id);
    assert!(closed, 73570005);

    // state of 1 means passed
    let state = aptos_governance::get_proposal_state(prop_id);
    assert!(state == 1, 73570006);

    // issue with transactions being atomic
    timestamp::fast_forward_seconds(1);

    // confirm that the voting would allow for a resolution
    // it cannot resolve because the transaction hashes (of this test script) do no match.
    let can_resolve = voting::check_resolvable_ex_hash<GovernanceProposal>(@ol_framework, prop_id);
    assert!(can_resolve, 73570007);
  }
}