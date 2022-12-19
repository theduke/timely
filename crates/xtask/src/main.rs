use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use xshell::cmd;

fn main() {
    match run() {
        Ok(_) => todo!(),
        Err(err) => {
            eprintln!("Command failed: {err:?}");
        }
    }
}

fn run() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args.cmd {
        SubCmd::Develop(c) => c.run(),
    }
}

fn root_dir() -> Result<PathBuf, anyhow::Error> {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .context("Missing required env var CARGO_MANIFEST_DIR")?
        .canonicalize()?
        .parent()
        .and_then(|p| p.parent())
        .context("Could not find root dir")
        .map(|p| p.to_owned())
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCmd,
}

#[derive(clap::Subcommand)]
enum SubCmd {
    Develop(CmdDevelop),
}

#[derive(Parser)]
struct CmdDevelop {}

impl CliCommand for CmdDevelop {
    fn run(self) -> Result<(), anyhow::Error> {
        let root = root_dir()?;

        let sh1 = xshell::Shell::new()?;
        sh1.change_dir(&root);
        let _handle = std::thread::spawn(move || {
            cmd!(
                sh1,
                "cargo watch -x 'build -p timely_server --target wasm32-wasi'"
            )
            .run()
        });

        let sh2 = xshell::Shell::new()?;
        sh2.change_dir(&root);

        let wasm_path = root.join("target/wasm32-wasi/debug/timely_server.wasm");
        let _out = cmd!(sh2, "wcgi_runner --env-all --watch {wasm_path}").run()?;

        Ok(())
    }
}

trait CliCommand {
    fn run(self) -> Result<(), anyhow::Error>;
}
