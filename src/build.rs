use std::{
    fs::{self, File},
    io,
    path::Path,
};

use anyhow::Result;
use reqwest::ClientBuilder;

const MULTIPLAYER_TOKEN: &str = "/* multiplayer.js code */";

pub async fn fetch_raw() -> Result<()> {
    let client = ClientBuilder::new()
        .user_agent("tetra-labs/0.1.0 (bots@joel.net.au)")
        .build()?;

    let paths: Vec<_> = include_str!("fetch-paths.txt").trim().lines().collect();

    let base = Path::new("build/source");
    fs::create_dir_all(base)?;
    for path in paths {
        let fs_path = base.join(match path {
            "/" => "index.html",
            x => &x[1..],
        });
        if fs_path.exists() {
            continue;
        }
        dbg!(&fs_path);

        let bytes = client
            .get(&format!("https://tetr.io{path}"))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        fs::create_dir_all(fs_path.parent().unwrap())?;
        fs::write(fs_path, bytes)?;
    }

    Ok(())
}

pub fn split_bundle() -> Result<()> {
    let js = fs::read_to_string("build/source/js/tetrio.js")?;
    let lines: Vec<_> = js.lines().collect();
    assert_eq!(lines.len(), 5);

    fs::create_dir_all("raw/js/split")?;
    write_opt("build/lib.js", lines[1])?;
    write_opt("build/const.js", lines[2])?;

    let (main0, x) = lines[3].split_once("ge.init(),").unwrap();
    let (multiplayer, main1) = x.split_once("class ot{").unwrap();
    let main = format!("{main0}ge.init() {MULTIPLAYER_TOKEN}; class ot{{{main1}");
    write_opt("build/main.js", main)?;
    write_opt("build/multiplayer.js", multiplayer)?;

    Ok(())
}

pub fn join_bundle() -> Result<String> {
    let mut bundle = String::new();
    bundle += &fs::read_to_string("build/lib.js")?;
    bundle += &fs::read_to_string("build/const.js")?;

    let multiplayer = fs::read_to_string("build/multiplayer.js")?;
    bundle += &fs::read_to_string("build/main.js")?
        .replace(MULTIPLAYER_TOKEN, &format!(", {multiplayer}"));

    Ok(bundle)
}

pub fn write_opt<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    if !path.as_ref().exists() {
        fs::write(path, contents)
    } else {
        Ok(())
    }
}
