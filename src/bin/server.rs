use aurora_kvm::{gui, server};
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Launch configuration GUI
    #[arg(long)]
    configure: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    if cli.configure {
        if let Err(e) = gui::run_gui() {
            eprintln!("GUI error: {}", e);
            std::process::exit(1);
        }
    } else {
        server::run(cli.port).await?;
    }

    Ok(())
}
