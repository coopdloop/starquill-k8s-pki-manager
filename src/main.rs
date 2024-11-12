mod app;
mod cert;
mod config;
mod types;
mod ui;
mod utils;

use app::CertManager;
use config::ClusterConfig;
use types::Args;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self};

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load config
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

    // Create app state
    let cert_manager = CertManager::new(config, args.debug);

    if args.debug {
        println!("Debug mode enabled");
        println!("Using config file: {}", args.config);
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

    // Handle any errors that occurred during execution
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
        return Err(err);
    }

    Ok(())
}
