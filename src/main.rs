// src/main.rs
mod app;
mod cert;
mod config;
mod discovery;
mod kubeconfig;
mod types;
mod ui;
mod utils;
mod web;
mod metrics;

use app::CertManager;
use config::ClusterConfig;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use discovery::CertificateDiscovery;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self},
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};
use ui::{LoadingState, OnboardingState, StepStatus};
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

async fn init_with_loading(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    args: &Args,
) -> io::Result<(Arc<RwLock<WebServerState>>, Arc<RwLock<CertManager>>)> {
    let mut loading_state = LoadingState::new();
    let mut config = ClusterConfig::default();
    // Initialize SSH cache
    let ssh_cache = Arc::new(RwLock::new(discovery::SSHConnectionCache::load()?));

    // Start periodic checking
    discovery::start_periodic_check(
        Arc::clone(&ssh_cache),
        config.remote_user.clone(),
        config.ssh_key_path.clone(),
    );

    // let mut ssh_cache = discovery::SSHConnectionCache::load()?;
    let mut failed_nodes = Vec::new();

    // Show initial loading screen
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    sleep(Duration::from_millis(100));

    // Check configuration
    loading_state.steps[0].1 = StepStatus::InProgress;
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    sleep(Duration::from_millis(100));

    match ClusterConfig::load_from_file(&args.config) {
        Ok(loaded_config) => {
            config = loaded_config;
            loading_state.next_step();
        }
        Err(_) => {
            // If config doesn't exist, show onboarding
            let mut onboarding = OnboardingState::new();
            config = run_onboarding(terminal, &mut onboarding)?;

            // Save config
            config.save_to_file(&args.config)?;
            loading_state.next_step();
        }
    }
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    sleep(Duration::from_millis(500));

    // Initialize web server
    loading_state.steps[1].1 = StepStatus::InProgress;
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    let web_state = Arc::new(RwLock::new(WebServerState::new(Some(args.port))));
    loading_state.next_step();
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;

    // Test control plane connection
    loading_state.steps[2].1 = StepStatus::InProgress;
    terminal.draw(|f| {
        ui::loading::render_ssh_status(
            f,
            &config.control_plane,
            &config.worker_nodes,
            &failed_nodes,
        );
    })?;

    // Use the cache in your initialization
    let mut cache = ssh_cache.write().unwrap();
    let mut connection_failed = false;
    if !discovery::verify_ssh_connection(
        &config.control_plane,
        &config.remote_user,
        &config.ssh_key_path,
        &mut cache,
    )
    .await?
    {
        failed_nodes.push(config.control_plane.clone());
        connection_failed = true;
    }

    // Test worker node connections
    for worker in &config.worker_nodes {
        if !discovery::verify_ssh_connection(
            worker,
            &config.remote_user,
            &config.ssh_key_path,
            &mut cache,
        )
        .await?
        {
            failed_nodes.push(worker.clone());
            connection_failed = true;
        }
        terminal.draw(|f| {
            ui::loading::render_ssh_status(
                f,
                &config.control_plane,
                &config.worker_nodes,
                &failed_nodes,
            );
        })?;
    }

    if connection_failed {
        loading_state.steps[2].1 = StepStatus::Warning("Some nodes are unreachable".to_string());
        terminal.draw(|f| {
            ui::loading::render_ssh_status(
                f,
                &config.control_plane,
                &config.worker_nodes,
                &failed_nodes,
            );
        })?;
        sleep(Duration::from_secs(1));

        // Only fail completely if control plane is unreachable
        if failed_nodes.contains(&config.control_plane) {
            // Cleanup and exit
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Control plane is unreachable",
            ));
        }
    }

    loading_state.next_step();
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    sleep(Duration::from_millis(500));

    // Initialize cert manager
    loading_state.steps[3].1 = StepStatus::InProgress;
    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;

    // Clone config before moving
    let config_for_manager = config.clone();
    let cert_manager = Arc::new(RwLock::new(CertManager::new(
        config_for_manager,
        args.debug,
        Arc::clone(&web_state),
    )));

    // Initialize certificates and load status
    {
        let mut manager = cert_manager.write().unwrap();

        if let Err(e) = manager.load_certificate_status() {
            manager.log(&format!(
                "Note: No previous certificate status found: {}",
                e
            ));
        }

        // Initialize async operations
        match manager.initialize().await {
            Ok(_) => {
                manager.log("Successfully initialized existing certificates and configuration");
                loading_state.next_step();
            }
            Err(e) => {
                manager.log(&format!(
                    "Warning: Initialization completed with some issues: {}",
                    e
                ));
                loading_state.steps[3].1 =
                    StepStatus::Warning(format!("Partial initialization: {}", e));
            }
        }
    }

    terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    sleep(Duration::from_secs(1));

    // Start periodic certificate verification
    let discovery = CertificateDiscovery::new();
    discovery
        .start_periodic_verification(
            vec![config.control_plane.clone()]
                .into_iter()
                .chain(config.worker_nodes.clone())
                .collect(),
            config.ssh_key_path.clone(),
        )
        .await;

    // loading_state.next_step();
    // terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;
    //
    // loading_state.steps[3].1 = StepStatus::InProgress;
    // terminal.draw(|f| ui::loading::render_loading(f, &loading_state))?;

    //TODO BELOW
    // Initialize certificates and load status
    // manager.log("Checking for existing certificates and configuration...");
    // if let Err(e) = manager.load_certificate_status() {
    //     manager.log(&format!(
    //         "Note: No previous certificate status found: {}",
    //         e
    //     ));
    // }


    Ok((web_state, cert_manager))
}

fn run_onboarding(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut OnboardingState,
) -> io::Result<ClusterConfig> {
    loop {
        terminal.draw(|f| ui::onboarding::render_onboarding(f, state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    if state.completed {
                        // Convert onboarding state to ClusterConfig
                        return Ok(ClusterConfig {
                            control_plane: state.fields[0].value.clone(),
                            worker_nodes: state.fields[1]
                                .value
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .collect(),
                            ssh_key_path: state.fields[2].value.clone(),
                            remote_user: state.fields[3].value.clone(),
                            remote_dir: "/etc/kubernetes/pki".to_string(), // Default value
                        });
                    } else {
                        state.fields[state.current_field].editing = true;
                    }
                }
                KeyCode::Tab => {
                    if !state.completed {
                        state.next_field();
                    }
                }
                KeyCode::Esc => {
                    state.completed = true;
                }
                KeyCode::Char(c) => {
                    if state.fields[state.current_field].editing {
                        state.fields[state.current_field].value.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if state.fields[state.current_field].editing {
                        state.fields[state.current_field].value.pop();
                    }
                }
                _ => {}
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Terminal initialization after background tasks are spawned
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize with loading screen
    let (web_state, cert_manager) = init_with_loading(&mut terminal, &args).await?;

    // Setup web state with cert manager reference
    {
        let mut state = web_state.write().unwrap();
        state.cert_manager = Some(Arc::clone(&cert_manager));
    }

    // Create shutdown channel and spawn web server
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let web_state_clone = Arc::clone(&web_state);
    let web_server = tokio::spawn(async move {
        web::start_web_server(web_state_clone, shutdown_rx).await;
    });

    // Run app
    let res = app::run_app(&mut terminal, Arc::clone(&cert_manager)).await;

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
