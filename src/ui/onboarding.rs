use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct OnboardingState {
    pub fields: Vec<OnboardingField>,
    pub current_field: usize,
    pub completed: bool,
}

pub struct OnboardingField {
    pub label: String,
    pub value: String,
    pub editing: bool,
}

impl OnboardingState {
    pub fn new() -> Self {
        Self {
            fields: vec![
                OnboardingField {
                    label: "Control Plane IP".to_string(),
                    value: String::new(),
                    editing: false,
                },
                OnboardingField {
                    label: "Worker Node IPs (comma-separated)".to_string(),
                    value: String::new(),
                    editing: false,
                },
                OnboardingField {
                    label: "SSH Key Path".to_string(),
                    value: String::new(),
                    editing: false,
                },
                OnboardingField {
                    label: "Remote User".to_string(),
                    value: String::new(),
                    editing: false,
                },
            ],
            current_field: 0,
            completed: false,
        }
    }

    pub fn next_field(&mut self) {
        self.fields[self.current_field].editing = false;
        self.current_field = (self.current_field + 1) % self.fields.len();
        self.fields[self.current_field].editing = true;
    }
}

pub fn render_onboarding(frame: &mut Frame, state: &OnboardingState) {
    let area = centered_rect(60, 40, frame.area());

    // Create a vector of constraints
    let mut constraints = vec![Constraint::Length(3)];
    // Add constraints for each field
    constraints.extend(state.fields.iter().map(|_| Constraint::Length(3)));
    // Add final constraint for help text
    constraints.push(Constraint::Length(3));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let title = Paragraph::new("Cluster Configuration")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    for (i, field) in state.fields.iter().enumerate() {
        let style = if i == state.current_field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let text = if field.editing {
            format!("{}: {}_", field.label, field.value)
        } else {
            format!("{}: {}", field.label, field.value)
        };

        let paragraph = Paragraph::new(text)
            .style(style)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(paragraph, chunks[i + 1]);
    }

    let help_text = if !state.completed {
        "Press Enter to edit field | Tab to move to next field | Esc to finish"
    } else {
        "Press Enter to save and continue"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[chunks.len() - 1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
