use libra_query::query_type::QueryType;
use libra_smoke_tests::libra_smoke::LibraSmoke;

/// Testing the query library
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn libra_view_test() {
    let mut s = LibraSmoke::new(None).await.expect("could not start swarm");

    let c = s.client();

    let q = QueryType::View {
        function_id: "0x1::gas_coin::supply".to_string(),
        type_args: None,
        args: None,
    };
    match q.query_to_json(Some(c.to_owned())).await {
        Ok(v) => {
            println!("v: {:?}", v);
            // let b: LibraBalanceDisplay = serde_json::from_value(v).unwrap();
            // assert!(b.unlocked == 10.0);
            // assert!(b.total == 10.0);
        }
        Err(e) => {
            println!("e: {:?}", e);
            panic!("nothing returned from query");
        }
    }
}
