#[macro_use]
extern crate log;

use std::{fs, path::Path};

// use actix_files::Files;
// use actix_web::{
//     body::MessageBody,
//     dev::{ServiceRequest, ServiceResponse},
//     middleware::{from_fn, Next},
//     App, Error, HttpServer,
// };
use anyhow::Result;
use clap::{Parser, Subcommand};
use glob::glob;

mod build;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Build,
    Patch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    match cli.command {
        Command::Build => {
            build::fetch_raw().await?;
            build::split_bundle()?;
        }
        Command::Patch => {
            // fs::remove_dir_all("dist")?;
            for res in glob("build/source/**/*")? {
                let path = res?;
                let path_name = path
                    .strip_prefix("build/source/")
                    .unwrap()
                    .to_str()
                    .unwrap();
                let path_out = format!("dist/{path_name}");
                dbg!(&path_out);
                let meta = fs::metadata(&path)?;
                if meta.is_file() {
                    let data = match path_name {
                        "index.html" => fs::read_to_string(path)?
                            .replace("/bootstrap.js", "/js/tetrio.js")
                            .into(),

                        "js/tetrio.js" => (build::join_bundle()?
                            .replace("if(_.domain)", "if(false)") // disable domain hijack check
                            .replace("sentry_enabled:!0", "sentry_enabled:false")
                            + include_str!("append.js"))
                        .into(),

                        "css/tetrio.css" => (fs::read_to_string(path)?
                            .replace("SigliaTripDisappear 5s 10s", "SigliaTripDisappear 2s 2s"))
                        .into_bytes(),

                        _ => fs::read(path)?,
                    };
                    fs::write(path_out, data)?;
                } else if meta.is_dir() {
                    fs::create_dir_all(path_out)?;
                }
            }
        }
    }

    Ok(())
}
