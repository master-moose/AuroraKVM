use aurora_kvm::{gui, server};
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Run in headless mode (no GUI, server only)
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    if cli.headless {
        // Run server only
        server::run(cli.port).await?;
    } else {
        // Default: Launch GUI with server running in background
        let port = cli.port;

        // Create connected clients state
        let connected_clients = aurora_kvm::connected::create_connected_clients();
        let connected_for_server = connected_clients.clone();

        // Spawn server in background
        tokio::spawn(async move {
            if let Err(e) = server::run_with_state(port, connected_for_server).await {
                eprintln!("Server error: {}", e);
            }
        });

        // Launch GUI (blocks until closed)
        if let Err(e) = gui::run_gui(Some(connected_clients)) {
            eprintln!("GUI error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
