use super::styles::*;
use crate::app::CertManager;
use crate::types::{ActiveSection, AppMode};
use crate::utils::constants::BACKGROUND_ART;
use ratatui::layout::Margin;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_all(f: &mut Frame, cert_manager: &CertManager) {
    let art = Paragraph::new(BACKGROUND_ART).style(Style::default().fg(Color::LightBlue));
    f.render_widget(art, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),      // Title
            Constraint::Length(7),      // Status/Config
            Constraint::Percentage(40), // Menu + Cert Status section
            Constraint::Percentage(40), // Logs + Trust Info section
            Constraint::Length(3),      // Help
        ])
        .split(f.area());

    render_title(f, chunks[0]);

    match cert_manager.mode {
        AppMode::EditConfig => {
            render_config_editor(f, chunks[1], cert_manager);
        }
        _ => {
            render_status(f, chunks[1], cert_manager);
        }
    }

    // Split middle section horizontally
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0) // Remove margin
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    render_menu(f, middle_chunks[0], cert_manager);
    render_certificate_status(f, middle_chunks[1], cert_manager);

    // Split bottom section horizontally
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0) // Remove margin
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[3]);

    render_logs(f, bottom_chunks[0], cert_manager);
    render_trust_info(f, bottom_chunks[1], cert_manager);

    render_help(f, chunks[4], &cert_manager.mode);

    // Render confirmation dialog on top if active
    if cert_manager.mode == AppMode::Confirmation {
        render_confirmation_dialog(f, f.area(), cert_manager);
    }
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![
        // First line with just Starquill
        Line::from(vec![Span::styled(
            "Starquill",
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD)
                .underlined(),
        )]),
        // Second line with Kubernetes Certificate Manager
        Line::from(vec![Span::styled(
            "Kubernetes Certificate Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(BORDER_STYLE),
    );
    f.render_widget(title, area);
}

fn render_status(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    let web_state = cert_manager.web_state.read().unwrap();
    let web_status = if web_state.is_running {
        let url = format!("http://localhost:{}", web_state.port);
        vec![
            Span::styled("Web UI: ", Style::default().fg(Color::Cyan)),
            Span::styled("Running", Style::default().fg(Color::Green)),
            Span::raw(" at "),
            Span::styled(
                url,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::raw(" (press O to open in browser)"),
        ]
    } else {
        vec![
            Span::styled("Web UI: ", Style::default().fg(Color::Cyan)),
            Span::styled("Starting...", Style::default().fg(Color::Yellow)),
        ]
    };

    let mut status_info = cert_manager.get_status_info();
    status_info.push(Line::from(web_status));

    let status = Paragraph::new(status_info)
        .block(
            Block::default()
                .title("Status")
                .title_style(TITLE_STYLE)
                .borders(Borders::ALL)
                .border_style(BORDER_STYLE),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(status, area);
}

fn render_menu(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    // Calculate visible items based on area height
    let visible_height = area.height.saturating_sub(2) as usize;
    let start_index = cert_manager.menu_scroll;

    let menu_items: Vec<ListItem> = cert_manager
        .menu_items
        .iter()
        .enumerate()
        .skip(start_index)
        .take(visible_height)
        .map(|(i, item)| {
            let base_style = if item == "Automate all" {
                AUTOMATE_STYLE
            } else if i == cert_manager.selected_menu {
                MENU_HIGHLIGHT_STYLE
            } else {
                MENU_STYLE
            };

            let prefix = if i == cert_manager.selected_menu {
                "> "
            } else {
                "  "
            };

            let final_style = if i == cert_manager.selected_menu && item == "Automate all" {
                AUTOMATE_SELECTED_STYLE
            } else {
                base_style
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, final_style),
                Span::styled(item, final_style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(format!(
            "Menu [{}/{}]",
            cert_manager.selected_menu + 1,
            cert_manager.menu_items.len()
        ))
        .title_style(TITLE_STYLE)
        .borders(Borders::ALL)
        .border_style(if cert_manager.active_section == ActiveSection::Menu {
            BORDER_STYLE.fg(Color::Cyan)
        } else {
            BORDER_STYLE
        });

    let menu_area = area.inner(Margin {
        vertical: 0,
        horizontal: 1, // Space for scrollbar
    });

    // Calculate scroll offset to keep selected item visible
    // let visible_items = (area.height as usize).saturating_sub(2); // Subtract 2 for borders
    // let scroll_offset = if cert_manager.selected_menu >= visible_items {
    //     cert_manager.selected_menu.saturating_sub(visible_items - 1)
    // } else {
    //     0
    // };

    let menu = List::new(menu_items).block(block);

    f.render_widget(menu, menu_area);

    // Add scrollbar
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(cert_manager.menu_items.len())
        .viewport_content_length(area.height.saturating_sub(2) as usize)
        .position(cert_manager.selected_menu);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_certificate_status(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    // let status_info = cert_manager.get_certificate_status_info();
    // let mut status_info = Vec::new();

    let visible_height = area.height.saturating_sub(3) as usize;
    let cert_info = cert_manager.get_certificate_status_info();
    let start_index = cert_manager.cert_status_scroll;

    let visible_certs = if cert_info.len() == 1
        && cert_info[0].spans[0].content == "No certificates generated yet"
    {
        cert_manager.get_certificate_status_info()
    } else {
        cert_info
            .iter()
            .skip(start_index)
            .take(visible_height)
            .cloned()
            .collect()
    };

    let block = Block::default()
        .title(format!(
            "Certificate Status [{}/{}]",
            start_index + 1,
            cert_info.len()
        ))
        .borders(Borders::ALL)
        .border_style(
            if cert_manager.active_section == ActiveSection::CertStatus {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            },
        );

    let status_widget = Paragraph::new(visible_certs).block(block);
    f.render_widget(status_widget, area);

    // Add scrollbar
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(cert_manager.menu_items.len())
        .viewport_content_length(area.height.saturating_sub(2) as usize)
        .position(cert_manager.cert_status_scroll);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

pub fn render_logs(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    let log_count = cert_manager.logs.len();
    let visible_height = (area.height as usize).saturating_sub(2); // Subtract 2 for borders
    let start_index = cert_manager.log_scroll;
    let end_index = (start_index + visible_height).min(log_count);

    let visible_logs: Vec<ListItem> = cert_manager
        .logs
        .iter()
        .skip(start_index)
        .take(visible_height)
        .map(|log| {
            let style = if log.contains("Error") {
                LOG_ERROR_STYLE
            } else if log.contains("failed") {
                LOG_ERROR_STYLE
            } else if log.contains("Successfully") {
                LOG_SUCCESS_STYLE
            } else if log.contains("successfully") {
                LOG_SUCCESS_STYLE
            } else if log.contains("[DEBUG]") {
                LOG_DEBUG_STYLE
            } else {
                LOG_INFO_STYLE
            };

            ListItem::new(Line::from(vec![Span::styled(log, style)]))
        })
        .collect();

    let scroll_indicator = if log_count > visible_height {
        format!(" [{}-{}/{}]", start_index + 1, end_index, log_count)
    } else {
        String::new()
    };

    let at_bottom = end_index == log_count;
    let scroll_style = if at_bottom {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let logs = List::new(visible_logs).block(
        Block::default()
            .title(Span::styled(
                format!("Logs{}", scroll_indicator),
                scroll_style,
            ))
            .borders(Borders::ALL)
            .border_style(if cert_manager.active_section == ActiveSection::Logs {
                Style::default().fg(Color::Cyan)
            } else {
                BORDER_STYLE
            }),
    );

    f.render_widget(logs, area);

    // Add scrollbar
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(cert_manager.menu_items.len())
        .viewport_content_length(area.height.saturating_sub(2) as usize)
        .position(cert_manager.log_scroll);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

pub fn render_config_editor(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    // First render the background ASCII art
    let art = Paragraph::new(BACKGROUND_ART).style(Style::default().fg(Color::DarkGray));
    f.render_widget(art, area);

    let items = vec![
        ("Remote User", 0),
        ("Control Plane IP", 1),
        ("Worker Node IPs (comma-separated)", 2),
        ("Remote Directory", 3),
        ("SSH Key Path", 4),
    ];

    let config_items: Vec<ListItem> = items
        .iter()
        .map(|(label, index)| {
            let style = if *index == cert_manager.config_editor.current_field {
                HIGHLIGHT_STYLE
            } else {
                BASE_STYLE
            };

            let value = if cert_manager.config_editor.is_editing
                && *index == cert_manager.config_editor.current_field
            {
                &cert_manager.config_editor.editing_value
            } else {
                &cert_manager.config_editor.fields[*index]
            };

            // Show completions if available
            let mut spans = vec![
                Span::styled(format!("{}: ", label), style),
                Span::styled(
                    value,
                    if *index == cert_manager.config_editor.current_field {
                        style.add_modifier(Modifier::UNDERLINED)
                    } else {
                        style
                    },
                ),
            ];

            // Add completion preview if available
            if *index == 4
                && cert_manager.config_editor.is_editing
                && !cert_manager.config_editor.completions.is_empty()
            {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!(
                        "(Tab: {})",
                        cert_manager.config_editor.completions
                            [cert_manager.config_editor.selected_completion]
                    ),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let block = Block::default()
        .title("Configuration Editor")
        .title_style(TITLE_STYLE)
        .borders(Borders::ALL)
        .border_style(ACTIVE_BORDER_STYLE);

    let _inner_area = block.inner(area);

    let list = List::new(config_items).block(block);

    f.render_widget(list, area);
}

pub fn render_help(f: &mut Frame, area: Rect, mode: &AppMode) {
    let help_text = match mode {
        AppMode::EditConfig => vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Edit | "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Complete | "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Exit Config"),
        ],
        AppMode::Confirmation => vec![
            Span::styled("Y", Style::default().fg(Color::Green)),
            Span::raw(": Confirm | "),
            Span::styled("N", Style::default().fg(Color::Red)),
            Span::raw("/"),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ],
        AppMode::Normal => vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Select | "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ],
    };

    let help_style = match mode {
        AppMode::Confirmation => Style::default().fg(Color::White),
        _ => Style::default().fg(Color::Gray),
    };

    let help = Paragraph::new(Line::from(help_text))
        .block(
            Block::default()
                .title("Help")
                .title_style(TITLE_STYLE)
                .borders(Borders::ALL)
                .border_style(if matches!(mode, AppMode::Confirmation) {
                    BORDER_STYLE.fg(Color::Cyan)
                } else {
                    BORDER_STYLE
                }),
        )
        .style(help_style)
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

pub fn render_confirmation_dialog(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    if let Some(dialog) = &cert_manager.confirmation_dialog {
        // Calculate dialog size and position
        let dialog_area = Rect {
            x: area.x + (area.width - 60) / 2,
            y: area.y + (area.height - 7) / 2,
            width: 60,
            height: 7,
        };

        // Render dialog background
        let dialog_block = Block::default()
            .title("Confirmation")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let text = vec![
            Line::from(vec![Span::raw(&dialog.message)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled(
                    "Y",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("es / "),
                Span::styled(
                    "N",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("o"),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(dialog_block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, dialog_area); // Clear the background
        f.render_widget(paragraph, dialog_area);
    }
}

pub fn render_trust_info(f: &mut Frame, area: Rect, cert_manager: &CertManager) {
    let block = Block::default()
        .title("Trust Validation")
        .borders(Borders::ALL)
        .border_style(if cert_manager.active_section == ActiveSection::TrustInfo {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content = if let Some(store) = &cert_manager.trust_store {
        let store_vec: Vec<_> = store.iter().collect();
        let mut lines = Vec::new();

        for (node, trust_info) in store_vec.iter().skip(cert_manager.trust_info_scroll) {
            // Node header
            lines.push(Line::from(vec![Span::styled(
                format!("Node: {}", node),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));

            // Trust chain status
            let chain_status_color = if trust_info.trust_chain_valid {
                Color::Green
            } else {
                Color::Red
            };
            lines.push(Line::from(vec![
                Span::raw("  Trust Chain: "),
                Span::styled(
                    format!(
                        "{}",
                        if trust_info.trust_chain_valid {
                            "Valid"
                        } else {
                            "Invalid"
                        }
                    ),
                    Style::default().fg(chain_status_color),
                ),
            ]));

            // Permissions status
            let perm_status_color = if trust_info.permissions_valid {
                Color::Green
            } else {
                Color::Red
            };
            lines.push(Line::from(vec![
                Span::raw("  Permissions: "),
                Span::styled(
                    format!(
                        "{}",
                        if trust_info.permissions_valid {
                            "Valid"
                        } else {
                            "Invalid"
                        }
                    ),
                    Style::default().fg(perm_status_color),
                ),
            ]));

            // Expiring certificates
            if !trust_info.expiring_soon.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "  Expiring Certificates:",
                    Style::default().fg(Color::Yellow),
                )]));
                for cert in &trust_info.expiring_soon {
                    lines.push(Line::from(vec![Span::raw(format!("    - {}", cert))]));
                }
            }

            // Last checked timestamp
            lines.push(Line::from(vec![
                Span::raw("  Last Checked: "),
                Span::styled(
                    trust_info.last_checked.to_rfc3339(),
                    Style::default().fg(Color::Gray),
                ),
            ]));
            lines.push(Line::from(vec![Span::raw("")])); // Add spacing between nodes
        }
        lines
    } else {
        vec![Line::from("Trust information not available")]
    };

    let paragraph = Paragraph::new(content);
    f.render_widget(paragraph, inner_area);

    if let Some(store) = &cert_manager.trust_store {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(store.len())
            .viewport_content_length(area.height.saturating_sub(2) as usize)
            .position(cert_manager.trust_info_scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}
