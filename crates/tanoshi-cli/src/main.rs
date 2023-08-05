extern crate log;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::Serialize;
use tanoshi_lib::prelude::SourceInfo;
use tanoshi_vm::{prelude::ExtensionManager, PLUGIN_EXTENSION};

const TARGET: &str = env!("TARGET");

#[derive(Parser)]
#[clap(version, about)]
struct Opts {
    #[clap(short, long, default_value = "./")]
    path: String,
    #[clap(subcommand)]
    subcmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate index.json
    GenerateJson,
}

#[derive(Debug, Serialize)]
struct SourceIndex {
    #[serde(flatten)]
    source: SourceInfo,
    rustc_version: String,
    lib_version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    match opts.subcmd {
        Command::GenerateJson => {
            let target_dir_path = PathBuf::new().join("output").join(TARGET);
            tokio::fs::create_dir_all(&target_dir_path).await?;

            let mut read_dir = tokio::fs::read_dir(&opts.path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let mut name = format!("{:?}", entry.file_name());
                name.remove(0);
                name.remove(name.len() - 1);

                if name.ends_with(PLUGIN_EXTENSION) {
                    #[cfg(target_os = "linux")]
                    let name = name.replace("lib", "");

                    tokio::fs::copy(entry.path(), &target_dir_path.join(name).as_path()).await?;
                }
            }

            let extension_manager = ExtensionManager::new(&target_dir_path);
            extension_manager.load_all().await?;
            let source_list = extension_manager.list().await?;

            let mut indexes = vec![];
            for source in source_list {
                let (rustc_version, lib_version) = extension_manager.get_version(source.id)?;
                indexes.push(SourceIndex {
                    source,
                    rustc_version,
                    lib_version,
                })
            }

            let json = serde_json::to_string(&indexes)?;
            tokio::fs::write(target_dir_path.join("index").with_extension("json"), json).await?;
        }
    }

    Ok(())
}
