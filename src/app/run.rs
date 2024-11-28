use super::CertManager;
use crate::types::{AppMode, ConfirmationCallback, ConfirmationDialog, ScrollDirection};
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::{Arc, RwLock};
use std::{
    io,
    time::{Duration, Instant},
};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cert_manager: Arc<RwLock<CertManager>>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(30);

    loop {
        let mut manager = cert_manager.write().unwrap();
        manager.process_pending_logs();

        terminal.draw(|f| ui::render_all(f, &manager))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        drop(manager);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Get a new lock for handling events
                let mut manager = cert_manager.write().unwrap();

                match manager.mode {
                    AppMode::ViewLogs => match key.code {
                        KeyCode::Esc => {
                            manager.mode = AppMode::Normal;
                        }
                        KeyCode::Up => {
                            manager.scroll_logs(ScrollDirection::Up);
                        }
                        KeyCode::Down => {
                            manager.scroll_logs(ScrollDirection::Down);
                        }
                        KeyCode::PageUp => {
                            manager.scroll_logs(ScrollDirection::PageUp);
                        }
                        KeyCode::PageDown => {
                            manager.scroll_logs(ScrollDirection::PageDown);
                        }
                        KeyCode::Home => {
                            manager.scroll_logs(ScrollDirection::Top);
                        }
                        KeyCode::End => {
                            manager.scroll_logs(ScrollDirection::Bottom);
                        }
                        _ => {}
                    },
                    AppMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            manager.mode = AppMode::ViewLogs;
                        }
                        KeyCode::Up => {
                            manager.selected_menu = manager
                                .selected_menu
                                .checked_sub(1)
                                .unwrap_or(manager.menu_items.len() - 1);
                        }
                        KeyCode::Down => {
                            manager.selected_menu =
                                (manager.selected_menu + 1) % manager.menu_items.len();
                        }
                        KeyCode::Char('o') => {
                            if key.modifiers == KeyModifiers::NONE {
                                manager.open_web_ui();
                            }
                        }
                        KeyCode::Enter => match manager.selected_menu {
                            0 => {
                                if let Err(e) = manager.generate_root_ca() {
                                    manager.log(&format!("Error: {}", e));
                                }
                            }
                            1 => {
                                if let Err(e) = manager.generate_kubernetes_cert() {
                                    manager.log(&format!("Error: {}", e));
                                }
                            }
                            2 => {
                                if let Err(e) = manager.generate_kubelet_client_cert() {
                                    manager.log(&format!("Error: {}", e));
                                }
                            }
                            3 => {
                                if let Err(e) = manager.generate_worker_node_certs() {
                                    manager.log(&format!("Error: {}", e));
                                }
                            }
                            4 => {
                                if let Err(e) = manager.generate_service_account_keys() {
                                    manager.log(&format!("Error: {}", e));
                                }
                            }
                            5 => {
                                manager.set_current_operation(
                                    "Generating Controller Manager Certificate",
                                );
                                if let Err(e) = manager.generate_controller_manager_cert() {
                                    manager.log(&format!(
                                        "Failed to generate Controller Manager certificate: {}",
                                        e
                                    ));
                                } else {
                                    manager.log(
                                        "Controller Manager certificate generated successfully",
                                    );
                                }
                            }

                            6 => {
                                // Generate Kubeconfigs
                                manager.set_current_operation("Starting kubeconfig generation...");
                                if let Err(e) = manager.generate_all_kubeconfigs() {
                                    manager.log(&format!("Failed to generate kubeconfigs: {}", e));
                                } else {
                                    manager.log("Kubeconfig generation completed successfully");
                                    // Offer to distribute
                                    manager.confirmation_dialog = Some(ConfirmationDialog {
                                        message:
                                            "Do you want to distribute the generated kubeconfigs?"
                                                .to_string(),
                                        callback: ConfirmationCallback::DistributePending,
                                    });
                                    manager.mode = AppMode::Confirmation;
                                }
                            }
                            7 => {
                                // Generate Encryption Config
                                manager.set_current_operation(
                                    "Starting encryption config generation...",
                                );
                                if let Err(e) = manager.generate_encryption_config() {
                                    manager.log(&format!(
                                        "Failed to generate encryption config: {}",
                                        e
                                    ));
                                } else {
                                    manager.log("Encryption config generated successfully");
                                    // Offer to distribute
                                    manager.confirmation_dialog = Some(ConfirmationDialog {
                                        message: "Do you want to distribute the encryption config?"
                                            .to_string(),
                                        callback: ConfirmationCallback::DistributePending,
                                    });
                                    manager.mode = AppMode::Confirmation;
                                }
                            }

                            8 => {
                                // Edit mode
                                manager.mode = AppMode::EditConfig;
                                manager.log("Entered configuration mode");
                            }
                            9 => {
                                // Save mode
                                if let Err(e) = manager.save_config() {
                                    manager.log(&format!("Failed to save config: {}", e));
                                } else {
                                    manager.log("Configuration saved successfully");
                                }
                            }
                            10 => {
                                // Verify Certificates
                                if let Err(e) = manager.verify_certificates() {
                                    manager.log(&format!("Certificate verification failed: {}", e));
                                }
                            }
                            11 => return Ok(()), // Exit
                            12 => {
                                // Distribute Pending Certificates
                                let undistributed = manager.cert_tracker.get_undistributed();
                                if undistributed.is_empty() {
                                    manager.log("No pending certificates to distribute");
                                } else {
                                    manager.confirmation_dialog = Some(ConfirmationDialog {
                                        message: format!(
                                            "Distribute {} pending certificates?",
                                            undistributed.len()
                                        ),
                                        callback: ConfirmationCallback::DistributePending,
                                    });
                                    manager.mode = AppMode::Confirmation;
                                }
                            }
                            13 => {
                                // Save Certificate Status
                                if let Err(e) = manager.save_certificate_status() {
                                    manager
                                        .log(&format!("Failed to save certificate status: {}", e));
                                } else {
                                    manager.log("Certificate status saved successfully");
                                }
                            }
                            14 => {
                                // Automate all
                                manager.confirmation_dialog = Some(ConfirmationDialog {
                                    message: "Do you want to automatically generate and distribute all certificates?".to_string(),
                                    callback: ConfirmationCallback::AutomateAll,
                                });
                                manager.mode = AppMode::Confirmation;
                            }

                            _ => manager.log("Function not implemented yet"),
                        },
                        _ => {}
                    },
                    AppMode::EditConfig => {
                        manager.handle_config_edit(key.code);
                    }

                    AppMode::Confirmation => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            manager.handle_confirmation(true);
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            manager.handle_confirmation(false);
                        }
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
