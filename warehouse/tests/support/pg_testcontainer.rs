use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::{thread, time::Duration};
use testcontainers::{clients::Cli, core::WaitFor, Container, GenericImage, RunnableImage};

// need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
// copy pasta from: https://www.reddit.com/r/rust/comments/1294pfy/help_testcontainers_client_does_not_live_long/?rdt=54538
static CLI: Lazy<Cli> = Lazy::new(Cli::default);

// Note: testcontainers drops the container once it's out of scope. So for each
// test we should pass it along, even if we don't reference it.
// Otherwise, the docker contain will stop before you run the test
pub async fn get_test_pool<'a>() -> anyhow::Result<(PgPool, Container<'a, GenericImage>)> {
    let container = start_container();
    // prepare connection string
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    let sqlx_pool = PgPool::connect(connection_string).await?;
    println!("database container started at: {}\n", &connection_string);

    Ok((sqlx_pool, container))
}

pub fn start_container<'a>() -> Container<'a, GenericImage> {
    let container = GenericImage::new("postgres", "17.0-alpine3.20")
        .with_env_var("POSTGRES_PASSWORD", "testing")
        .with_env_var("POSTGRES_HOST_AUTH_METHOD".to_owned(), "trust".to_owned())
        .with_env_var("POSTGRES_DB".to_owned(), "postgres".to_owned())
        .with_env_var("POSTGRES_USER".to_owned(), "postgres".to_owned())
        .with_env_var("POSTGRES_PASSWORD".to_owned(), "postgres".to_owned())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ));

    let image = RunnableImage::from(container);
    // need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
    let container = CLI.run(image);
    container.start();
    // TODO: not sure why we need a bit of a wait since we have the WaitFor above
    // will otherwise get: "unexpected response from SSLRequest: 0x00 (sqlx_postgres::connection::tls:97)"
    thread::sleep(Duration::from_millis(500));

    container
}

#[tokio::test]
async fn test_meta_setup() {
    let (db, _c) = get_test_pool()
        .await
        .expect("test could not start container");

    let query = sqlx::query("SELECT 'hello world!'")
        .execute(&db)
        .await
        .unwrap();
    assert!(query.rows_affected() == 1);
}
