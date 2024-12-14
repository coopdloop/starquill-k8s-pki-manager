// src/ui/loading.rs

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::LoadingState;

pub fn render_loading(f: &mut Frame, state: &LoadingState) {
    let size = f.area();

    // Create a centered box that's 80% of the screen width
    let width = (size.width as f32 * 0.8) as u16;
    let height = state.steps.len() as u16 + 4; // Add padding
    let x = (size.width - width) / 2;
    let y = (size.height - height) / 2;

    let loading_area = Rect::new(x, y, width, height);

    let loading_block = Block::default()
        .borders(Borders::ALL)
        .title(" Initializing Certificate Manager ");

    let lines: Vec<Line> = state
        .steps
        .iter()
        .map(|(step, status)| {
            let (symbol, color) = status.get_symbol_and_color();
            Line::from(vec![
                Span::styled(
                    format!("{} ", symbol),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(step),
            ])
        })
        .collect();

    let loading_text = Paragraph::new(lines)
        .block(loading_block)
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(loading_text, loading_area);
}

// Helper function to render SSH connection status
pub fn render_ssh_status(
    f: &mut Frame,
    control_plane: &str,
    worker_nodes: &[String],
    failed_nodes: &[String],
) -> Rect {
    let size = f.area();

    // Create a box that's 80% of the screen width
    let width = (size.width as f32 * 0.8) as u16;
    let height = (2 + worker_nodes.len()) as u16 + 2; // Header + control plane + workers + borders
    let x = (size.width - width) / 2;
    let y = (size.height - height) / 2;

    let area = Rect::new(x, y, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SSH Connection Status ");

    let mut lines = vec![];

    // Add control plane status
    let cp_status = if failed_nodes.contains(&control_plane.to_string()) {
        ("✗", Color::Red)
    } else {
        ("✓", Color::Green)
    };

    lines.push(Line::from(vec![
        Span::raw("Control Plane: "),
        Span::raw(control_plane),
        Span::styled(
            format!(" [{}]", cp_status.0),
            Style::default()
                .fg(cp_status.1)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Add worker node status
    for worker in worker_nodes {
        let status = if failed_nodes.contains(worker) {
            ("✗", Color::Red)
        } else {
            ("✓", Color::Green)
        };

        lines.push(Line::from(vec![
            Span::raw("Worker Node:  "),
            Span::raw(worker),
            Span::styled(
                format!(" [{}]", status.0),
                Style::default().fg(status.1).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let text = Paragraph::new(lines).block(block);
    f.render_widget(text, area);
    area
}

