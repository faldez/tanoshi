extern crate log;

mod data;
mod generate;
mod test;

use clap::{AppSettings, Clap};
use tanoshi_vm::{bus::ExtensionBus, vm};

#[derive(Clap)]
#[clap(version = "0.1.1", author = "Muhammad Fadhlika <fadhlika@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long)]
    path: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[cfg(not(feature = "disable-compiler"))]
    Compile,
    GenerateJson,
    Test(TestOption),
}

#[derive(Clap)]
struct TestOption {
    #[clap(long)]
    selector: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let extension_path = match opts.path {
        Some(path) => path,
        None => "target/wasm32-wasi/release".to_string(),
    };

    let (_, extension_tx) = vm::start();
    #[cfg(not(feature = "disable-compiler"))]
    if !matches!(opts.subcmd, SubCommand::Compile) {
        vm::load(&extension_path, extension_tx.clone()).await?;
    }

    #[cfg(feature = "disable-compiler")]
    vm::load(&extension_path, extension_tx.clone()).await?;

    let extension_bus = ExtensionBus::new("target/wasm32-wasi/release".to_string(), extension_tx);

    match opts.subcmd {
        #[cfg(not(feature = "disable-compiler"))]
        SubCommand::Compile => {
            let triples = [
                "x86_64-unknown-linux-gnu",
                "aarch64-unknown-linux-gnu",
                "x86_64-apple-darwin",
                "x86_64-pc-windows-msvc",
            ];
            for triple in triples {
                vm::compile_with_target(&extension_path, triple).await?;
            }
        }
        SubCommand::GenerateJson => {
            generate::generate_json(extension_bus).await?;
        }
        SubCommand::Test(config) => {
            test::test(extension_bus, config.selector).await?;
        }
    }

    Ok(())
}
