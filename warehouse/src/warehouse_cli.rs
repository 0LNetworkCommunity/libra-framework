use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use neo4rs::Graph;
use std::path::PathBuf;

use crate::{
    load_entrypoint::{ingest_all, try_load_one_archive},
    neo4j_init::{get_credentials_from_env, PASS_ENV, URI_ENV, USER_ENV},
    scan::scan_dir_archive,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// DB tools e.g.: backup, restore, export to json
pub struct WarehouseCli {
    #[clap(long, short('d'))]
    /// URI of graphDB e.g. neo4j+s://localhost:port
    db_uri: Option<String>,
    /// username of db
    db_username: Option<String>,
    /// db password
    db_password: Option<String>,

    #[clap(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Sub {
    /// scans sub directories for archive bundles
    IngestAll {
        #[clap(long, short('d'))]
        start_path: PathBuf,
    },
    /// process and load a single archive
    LoadOne {
        #[clap(long, short('d'))]
        archive_dir: PathBuf,
    },
    /// check archive is valid and can be decoded
    Check {
        #[clap(long, short('d'))]
        archive_dir: PathBuf,
    },
}

impl WarehouseCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Sub::IngestAll { start_path } => {
                let map = scan_dir_archive(start_path)?;
                let pool = try_db_connection_pool(self).await?;

                ingest_all(&map, &pool).await?;
            }
            Sub::LoadOne { archive_dir } => match scan_dir_archive(archive_dir)?.0.get(archive_dir)
            {
                Some(man) => {
                    let pool = try_db_connection_pool(self).await?;
                    try_load_one_archive(man, &pool).await?;
                }
                None => {
                    bail!(format!(
                        "ERROR: cannot find .manifest file under {}",
                        archive_dir.display()
                    ));
                }
            },
            Sub::Check { archive_dir } => match scan_dir_archive(archive_dir)?.0.get(archive_dir) {
                Some(_) => todo!(),
                None => {
                    bail!(format!(
                        "ERROR: cannot find .manifest file under {}",
                        archive_dir.display()
                    ));
                }
            },
        };
        Ok(())
    }
}

pub async fn try_db_connection_pool(cli: &WarehouseCli) -> Result<Graph> {
    let db = match get_credentials_from_env() {
        Ok((uri, user, password)) => Graph::new(uri, user, password).await?,
        Err(_) => {
            if cli.db_uri.is_some() && cli.db_username.is_some() && cli.db_password.is_some() {
                Graph::new(
                    cli.db_uri.as_ref().unwrap(),
                    cli.db_username.as_ref().unwrap(),
                    cli.db_password.as_ref().unwrap(),
                )
                .await?
            } else {
                println!("Must pass DB credentials, either with CLI args or environment variable");
                println!("call with --db-uri, --db-user, and --db-password");
                println!(
                    "Alternatively export credentials to env variables: {}, {}, {}",
                    URI_ENV, USER_ENV, PASS_ENV
                );
                bail!("could not get a db instance with credentials");
            }
        }
    };
    Ok(db)
}
