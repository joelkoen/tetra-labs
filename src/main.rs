#[macro_use]
extern crate log;

use std::{collections::BTreeMap, fs};

// use actix_files::Files;
// use actix_web::{
//     body::MessageBody,
//     dev::{ServiceRequest, ServiceResponse},
//     middleware::{from_fn, Next},
//     App, Error, HttpServer,
// };
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use regex::Regex;

mod build;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Build,
    Patch { include_multiplayer: Option<bool> },
    Reverse,
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
        Command::Patch {
            include_multiplayer,
        } => {
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

                        "js/tetrio.js" => {
                            (build::join_bundle(include_multiplayer.unwrap_or(true))?
                                .replace("if(_.domain)", "if(false)") // disable domain hijack check
                                .replace("sentry_enabled:!0", "sentry_enabled:false")
                                + include_str!("append.js"))
                            .into()
                        }

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
        Command::Reverse => {
            let input = fs::read_to_string("build/multiplayer2.cleaned.js")?.replace("- -", "+");

            let mut indent = Vec::new();
            for line in input.lines() {
                indent.push((line.len() - line.trim_start_matches(' ').len()) / 4);
            }

            let mut funs = BTreeMap::new();
            let mut iter = input.lines().enumerate();
            while let Some((i, line)) = iter.next() {
                if line.trim().starts_with("function ")
                    && line.chars().filter(|x| *x == ',').count() == 4
                {
                    let mut fun = String::new();
                    fun.push_str(line);
                    while let Some((_, line)) = iter.next() {
                        fun.push_str(line);
                        if line.trim() == "}" {
                            break;
                        }
                    }
                    funs.insert(i, fun);
                }
            }

            let mut dump = "let dump = {};function Ke(a,b) { return [a,b]; }".to_string();
            let expr = Regex::new(
                r#"[\w$]+\((?:'.{1,4}'|-?\d{1,4}), (?:'.{1,4}'|-?\d{1,4}), (?:'.{1,4}'|-?\d{1,4}), (?:'.{1,4}'|-?\d{1,4}), (?:'.{1,4}'|-?\d{1,4})\)"#,
            )?;
            let mut prev_indent = 0;
            for (i, line) in input.lines().enumerate() {
                let indent = indent.get(i).cloned().unwrap_or_default() as i32;
                for _ in 0..indent {
                    dump.push_str("    ");
                }
                let indent_diff = indent - prev_indent;
                if indent_diff != 0 {
                    let pattern = if indent_diff > 0 { "(()=>{" } else { "})();" };
                    for _ in 0..(indent_diff.abs()) {
                        dump.push_str(&pattern);
                    }
                }
                prev_indent = indent;

                if let Some(fun) = funs.get(&i) {
                    dump.push_str(fun);
                }

                for hit in expr.find_iter(&line) {
                    let hit = hit.as_str();
                    dump.push_str(&format!(r#"dump["{hit}"] = {hit}; /* line {i} */"#))
                }
                dump.push('\n');
            }
            dump.push_str("console.log(JSON.stringify(dump))");
            fs::write("build/so_much_fun.js", dump)?;

            let out = fs::read_to_string("build/so_much_fun_out.txt")?;
            let mut x = input.clone();
            for y in out.trim().lines() {
                let (a, b) = y.split_once("!!!").unwrap();
                x = x.replace(a, b);
            }
            fs::write("build/multiplayer.out.js", x)?;
        }
    }

    Ok(())
}
