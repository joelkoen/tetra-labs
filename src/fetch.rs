use std::{fs, path::Path};

use anyhow::Result;
use reqwest::ClientBuilder;

pub async fn run() -> Result<()> {
    let client = ClientBuilder::new()
        .user_agent("tetra-labs/0.1.0 (bots@joel.net.au)")
        .build()?;

    let paths: Vec<_> = include_str!("fetch-paths.txt").trim().lines().collect();

    let base = Path::new("raw");
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
