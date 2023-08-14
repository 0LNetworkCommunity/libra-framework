script {
  use aptos_framework::aptos_governance;
  use std::vector;

  fun main(proposal_id: u64){
      let next_hash = vector::empty();
      let _framework_signer = aptos_governance::resolve_multi_step_proposal(proposal_id, @0000000000000000000000000000000000000000000000000000000000000001, next_hash);
  }
}
