use super::CertManager;
use crate::types::{AppMode, ConfirmationCallback, ConfirmationDialog, ScrollDirection};
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut cert_manager: CertManager,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(30);

    loop {
        cert_manager.process_pending_logs();

        terminal.draw(|f| ui::render_all(f, &cert_manager))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match cert_manager.mode {
                    AppMode::ViewLogs => match key.code {
                        KeyCode::Esc => {
                            cert_manager.mode = AppMode::Normal;
                        }
                        KeyCode::Up => {
                            cert_manager.scroll_logs(ScrollDirection::Up);
                        }
                        KeyCode::Down => {
                            cert_manager.scroll_logs(ScrollDirection::Down);
                        }
                        KeyCode::PageUp => {
                            cert_manager.scroll_logs(ScrollDirection::PageUp);
                        }
                        KeyCode::PageDown => {
                            cert_manager.scroll_logs(ScrollDirection::PageDown);
                        }
                        KeyCode::Home => {
                            cert_manager.scroll_logs(ScrollDirection::Top);
                        }
                        KeyCode::End => {
                            cert_manager.scroll_logs(ScrollDirection::Bottom);
                        }
                        _ => {}
                    },
                    AppMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            cert_manager.mode = AppMode::ViewLogs;
                        }
                        KeyCode::Up => {
                            cert_manager.selected_menu = cert_manager
                                .selected_menu
                                .checked_sub(1)
                                .unwrap_or(cert_manager.menu_items.len() - 1);
                        }
                        KeyCode::Down => {
                            cert_manager.selected_menu =
                                (cert_manager.selected_menu + 1) % cert_manager.menu_items.len();
                        }
                        KeyCode::Enter => match cert_manager.selected_menu {
                            0 => {
                                if let Err(e) = cert_manager.generate_root_ca() {
                                    cert_manager.log(&format!("Error: {}", e));
                                }
                            }
                            1 => {
                                if let Err(e) = cert_manager.generate_kubernetes_cert() {
                                    cert_manager.log(&format!("Error: {}", e));
                                }
                            }
                            2 => {
                                if let Err(e) = cert_manager.generate_kubelet_client_cert() {
                                    cert_manager.log(&format!("Error: {}", e));
                                }
                            }
                            3 => {
                                if let Err(e) = cert_manager.generate_worker_node_certs() {
                                    cert_manager.log(&format!("Error: {}", e));
                                }
                            }
                            4 => {
                                if let Err(e) = cert_manager.generate_service_account_keys() {
                                    cert_manager.log(&format!("Error: {}", e));
                                }
                            }
                            5 => {
                                cert_manager.set_current_operation(
                                    "Generating Controller Manager Certificate",
                                );
                                if let Err(e) = cert_manager.generate_controller_manager_cert() {
                                    cert_manager.log(&format!(
                                        "Failed to generate Controller Manager certificate: {}",
                                        e
                                    ));
                                } else {
                                    cert_manager.log(
                                        "Controller Manager certificate generated successfully",
                                    );
                                }
                            }
                            6 => {
                                cert_manager.mode = AppMode::EditConfig;
                                cert_manager.log("Entered configuration mode");
                            }
                            7 => {
                                if let Err(e) = cert_manager.save_config() {
                                    cert_manager.log(&format!("Failed to save config: {}", e));
                                } else {
                                    cert_manager.log("Configuration saved successfully");
                                }
                            }
                            8 => {
                                // Verify Certificates
                                if let Err(e) = cert_manager.verify_certificates() {
                                    cert_manager
                                        .log(&format!("Certificate verification failed: {}", e));
                                }
                            }
                            9 => return Ok(()), // Exit
                            10 => {
                                // Distribute Pending Certificates
                                let undistributed = cert_manager.cert_tracker.get_undistributed();
                                if undistributed.is_empty() {
                                    cert_manager.log("No pending certificates to distribute");
                                } else {
                                    cert_manager.confirmation_dialog = Some(ConfirmationDialog {
                                        message: format!(
                                            "Distribute {} pending certificates?",
                                            undistributed.len()
                                        ),
                                        callback: ConfirmationCallback::DistributePending,
                                    });
                                    cert_manager.mode = AppMode::Confirmation;
                                }
                            }
                            11 => {
                                // Save Certificate Status
                                if let Err(e) = cert_manager.save_certificate_status() {
                                    cert_manager
                                        .log(&format!("Failed to save certificate status: {}", e));
                                } else {
                                    cert_manager.log("Certificate status saved successfully");
                                }
                            }
                            12 => {
                                // Automate all
                                cert_manager.confirmation_dialog = Some(ConfirmationDialog {
                                    message: "Do you want to automatically generate and distribute all certificates?".to_string(),
                                    callback: ConfirmationCallback::AutomateAll,
                                });
                                cert_manager.mode = AppMode::Confirmation;
                            }

                            _ => cert_manager.log("Function not implemented yet"),
                        },
                        _ => {}
                    },
                    AppMode::EditConfig => {
                        cert_manager.handle_config_edit(key.code);
                    }

                    AppMode::Confirmation => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            cert_manager.handle_confirmation(true);
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            cert_manager.handle_confirmation(false);
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
