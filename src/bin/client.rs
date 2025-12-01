use aurora_kvm::client;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Server host (auto-discovers if not specified)
    #[arg(short = 'H', long)]
    host: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    let host = if let Some(h) = cli.host {
        h
    } else {
        // Auto-discover servers
        println!("Discovering servers on network...");
        match aurora_kvm::discovery::discover_servers(5).await {
            Ok(servers) if !servers.is_empty() => {
                if servers.len() == 1 {
                    let (announcement, ip) = &servers[0];
                    let discovered = format!("{}:{}", ip, announcement.port);
                    println!("Found server: {} at {}", announcement.name, discovered);
                    discovered
                } else {
                    println!("\nFound {} servers:", servers.len());
                    for (i, (announcement, ip)) in servers.iter().enumerate() {
                        println!(
                            "  {}. {} at {}:{}",
                            i + 1,
                            announcement.name,
                            ip,
                            announcement.port
                        );
                    }
                    println!("\nUsing first server");
                    let (announcement, ip) = &servers[0];
                    format!("{}:{}", ip, announcement.port)
                }
            }
            Ok(_) => {
                eprintln!("No servers found. Please specify --host manually.");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Discovery error: {}. Please specify --host manually.", e);
                std::process::exit(1);
            }
        }
    };

    client::run(host).await?;

    Ok(())
}
