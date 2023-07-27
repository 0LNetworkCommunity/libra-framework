script {
use aptos_framework::aptos_governance;

fun main(proposal_id: u64){
    let _framework_signer = aptos_governance::resolve(proposal_id, @0000000000000000000000000000000000000000000000000000000000000001);
  }
}
