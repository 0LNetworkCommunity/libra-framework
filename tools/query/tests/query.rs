use libra_query::query_type::QueryType;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_types::move_resource::gas_coin::LibraBalanceDisplay;

/// Testing the query library
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn libra_query_test() {
    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start swarm");
    let val_acct = s.first_account.address();

    let c = s.client();

    let q = QueryType::Balance { account: val_acct };
    match q.query_to_json(&c).await {
        Ok(v) => {
            println!("v: {:?}", v);
            let b: LibraBalanceDisplay = serde_json::from_value(v).unwrap();
            assert_eq!(b.unlocked, 1000.0);
            assert_eq!(b.total, 1000.0);
        }
        Err(e) => {
            println!("e: {:?}", e);
            panic!("nothing returned from query");
        }
    }
}

/// test account struct annotation
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn account_annotate_test() {
    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start swarm");
    let val_acct = s.first_account.address();

    let c = s.client();

    let q = QueryType::Annotate { account: val_acct };
    let res = q.query_to_json(&c).await.unwrap();
    println!("{:#}", &res.as_str().unwrap());
    assert!(res.as_str().unwrap().contains("drop"));
}
