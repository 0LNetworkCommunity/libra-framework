use libra_warehouse::age_init::{cypher_template, init_age_db};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use testcontainers::{clients::Cli, core::WaitFor, Container, GenericImage, RunnableImage};

// need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
// copy pasta from: https://www.reddit.com/r/rust/comments/1294pfy/help_testcontainers_client_does_not_live_long/?rdt=54538
static CLI: Lazy<Cli> = Lazy::new(Cli::default);

static DB_NAME: &str = "warehouse_db";
static DB_USER: &str = "my_user";
static DUMMY_PASSWORD: &str = "dummy";

// Note: testcontainers drops the container once it's out of scope. So for each
// test we should pass it along, even if we don't reference it.
// Otherwise, the docker contain will stop before you run the test
pub async fn get_test_age_pool<'a>() -> anyhow::Result<(PgPool, Container<'a, GenericImage>)> {
    let container = start_age_container();
    // prepare connection string
    let connection_string = &format!(
        "postgres://{DB_USER}:{DUMMY_PASSWORD}@127.0.0.1:{}/{DB_NAME}",
        container.get_host_port_ipv4(5432)
    );

    let sqlx_pool = PgPool::connect(connection_string).await?;
    println!("database container started at: {}\n", &connection_string);

    Ok((sqlx_pool, container))
}

pub fn start_age_container<'a>() -> Container<'a, GenericImage> {
    let container = GenericImage::new("apache/age", "release_PG16_1.5.0")
        .with_env_var("POSTGRES_DB".to_owned(), DB_NAME.to_owned())
        .with_env_var("POSTGRES_USER".to_owned(), DB_USER.to_owned())
        .with_env_var("POSTGRES_PASSWORD".to_owned(), DUMMY_PASSWORD.to_owned())
        .with_wait_for(WaitFor::message_on_stdout(
            // "database system is ready to accept connections",
            "PostgreSQL init process complete; ready for start up.",
        ));

    let image = RunnableImage::from(container);
    // need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
    let container = CLI.run(image);
    container.start();

    container
}

#[tokio::test]
async fn test_age_meta_setup() {
    let (pool, _c) = get_test_age_pool()
        .await
        .expect("test could not start container");
    init_age_db(&pool).await.unwrap();

    let query = sqlx::query("SELECT 'hello world!'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(query.rows_affected() == 1);

    let query = sqlx::query(&cypher_template(
        r#"
          CREATE (a:label)
        "#,
    ))
    .execute(&pool)
    .await
    .unwrap();

    // TODO: unclear why rows affected is 0, but it looks like the correct behavior
    assert!(query.rows_affected() == 0);
}
