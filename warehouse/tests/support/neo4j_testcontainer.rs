use once_cell::sync::Lazy;
use testcontainers::{clients::Cli, core::WaitFor, Container, GenericImage, RunnableImage};

// need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
// copy pasta from: https://www.reddit.com/r/rust/comments/1294pfy/help_testcontainers_client_does_not_live_long/?rdt=54538
static CLI: Lazy<Cli> = Lazy::new(Cli::default);

// Note: testcontainers drops the container once it's out of scope. So for each
// test we should pass it along, even if we don't reference it.
// Otherwise, the docker contain will stop before you run the test

pub fn start_neo4j_container<'a>() -> Container<'a, GenericImage> {
    let container = GenericImage::new("neo4j", "5.25.1-community")
        .with_env_var("NEO4J_AUTH".to_owned(), "none".to_owned())
        // NOTE: To included data science modules in neo4j use:
        // .with_env_var("NEO4J_PLUGINS".to_owned(), r#"["graph-data-science"]"#);
        .with_wait_for(WaitFor::message_on_stdout("Started."));


    let image = RunnableImage::from(container);
    // need to wrap the docker cli in a once_cell so that the borrow does not cause issues when container is passed along
    let container = CLI.run(image);
    container.start();

    container
}

#[tokio::test]
async fn test_neo4j_meta_setup() {
    let _container = start_neo4j_container();
}
