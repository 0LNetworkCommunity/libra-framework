use libra_query::query_type::QueryType;
use libra_smoke_tests::libra_smoke::LibraSmoke;

/// Testing the query library
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn libra_view_test() {
    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start swarm");

    let c = s.client();

    let q = QueryType::View {
        function_id: "0x1::libra_coin::supply".to_string(),
        type_args: None,
        args: None,
    };
    match q.query_to_json(Some(c.to_owned())).await {
        Ok(v) => {
            println!("v: {:?}", v);
        }
        Err(e) => {
            println!("e: {:?}", e);
            panic!("nothing returned from query");
        }
    }
}
