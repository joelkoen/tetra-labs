#[macro_use]
extern crate log;

use std::fs;

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

mod fetch;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Fetch,
    Patch,
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    match cli.command {
        Command::Fetch => {
            fetch::run().await?;
        }
        Command::Patch => {
            // fs::remove_dir_all("dist")?;
            for res in glob("raw/**/*")? {
                let path = res?;
                let path_name = path.strip_prefix("raw/").unwrap().to_str().unwrap();
                let path_out = format!("dist/{path_name}");
                dbg!(&path_out);
                let meta = fs::metadata(&path)?;
                if meta.is_file() {
                    let data = match path_name {
                        "index.html" => fs::read_to_string(path)?
                            .replace("/bootstrap.js", "/js/tetrio.js")
                            .into(),

                        "js/tetrio.js" => (fs::read_to_string(path)?
                            .replace("if(_.domain)", "if(false)") // disable domain hijack check
                            .replace("sentry_enabled:!0", "sentry_enabled:false")
                            .replace(r#"(kt[r(0,0,0,2687,")#]6)")"#, "") // disable debugger trap
                            + include_str!("append.js"))
                        .into(),

                        _ => fs::read(path)?,
                    };
                    fs::write(path_out, data)?;
                } else if meta.is_dir() {
                    fs::create_dir_all(path_out)?;
                }
            }
        }
        Command::Serve => {
            // HttpServer::new(|| {
            //     App::new()
            //         .service(Files::new("/", "dist").index_file("index.html"))
            //         .wrap(from_fn(middleware))
            // })
            // .bind(("127.0.0.1", 8000))?
            // .run()
            // .await?;
        }
    }

    Ok(())
}
