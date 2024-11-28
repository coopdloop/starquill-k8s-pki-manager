mod app;
mod cert;
mod config;
mod types;
mod ui;
mod utils;
mod web;
mod kubeconfig;

use app::CertManager;
use config::ClusterConfig;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self},
    sync::{Arc, RwLock},
    time::Duration,
};
use web::WebServerState;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value = "cluster_config.json")]
    pub config: String,
    #[arg(short, long)]
    pub debug: bool,
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Load ClusterConfig state
    let config = match ClusterConfig::load_from_file(&args.config) {
        Ok(config) => {
            if args.debug {
                eprintln!("Loaded configuration from: {}", args.config);
            }
            config
        }
        Err(e) => {
            eprintln!("Error loading config: {}. Using defaults.", e);
            ClusterConfig::default()
        }
    };

    let web_state = Arc::new(RwLock::new(WebServerState::new(Some(args.port))));

    // Create CertManager with lock tracking
    let cert_manager = {
        let manager = Arc::new(RwLock::new(CertManager::new(
            config,
            args.debug,
            Arc::clone(&web_state),
        )));
        manager
    };

    {
        let mut state = web_state.write().unwrap();
        state.cert_manager = Some(Arc::clone(&cert_manager));
    }

    // Create a shutdown signal channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // Now spawn web server after CertManager is set
    let web_state_clone = Arc::clone(&web_state);
    let web_server = tokio::spawn(async move {
        web::start_web_server(web_state_clone, shutdown_rx).await;
    });

    // Small delay to ensure web server starts
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if args.debug {
        eprintln!("Debug mode enabled");
        eprintln!("Using config file: {}", args.config);
    }

    // Run app
    let res = app::run_app(&mut terminal, cert_manager);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Send shutdown signal to web server
    let _ = shutdown_tx.send(());

    // Wait for web server to shutdown
    let _ = web_server.await;

    // Handle any errors that occurred during execution
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
        return Err(err);
    }

    // Final lock check
    if ACTIVE_LOCKS.load(Ordering::SeqCst) > 0 {
        eprintln!(
            "Warning: {} locks still active at shutdown",
            ACTIVE_LOCKS.load(Ordering::SeqCst)
        );
    }

    Ok(())
}

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ACTIVE_LOCKS: AtomicUsize = AtomicUsize::new(0);

pub fn track_lock_count(delta: isize, location: &str) {
    let new_count = if delta > 0 {
        ACTIVE_LOCKS.fetch_add(delta as usize, Ordering::SeqCst)
    } else {
        ACTIVE_LOCKS.fetch_sub((-delta) as usize, Ordering::SeqCst)
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    eprintln!(
        "[{}ms] {} - Lock count: {} (delta: {})",
        timestamp, location, new_count, delta
    );
}
