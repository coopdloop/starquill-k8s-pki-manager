// src/ui/mod.rs
pub(crate) mod loading;
pub(crate) mod onboarding;
mod render;
mod styles;


#[derive(Clone, Debug)]
pub struct LoadingState {
    pub steps: Vec<(String, StepStatus)>,
    current_step: usize,
}

#[derive(Clone, Debug)]
pub enum StepStatus {
    Pending,
    InProgress,
    Complete,
    Warning(String),
    Failed(String),
}

impl StepStatus {
    pub fn get_symbol_and_color(&self) -> (&str, ratatui::style::Color) {
        use ratatui::style::Color;
        match self {
            StepStatus::Pending => ("○", Color::DarkGray),
            StepStatus::InProgress => ("◐", Color::Yellow),
            StepStatus::Complete => ("●", Color::Green),
            StepStatus::Warning(_) => ("●", Color::LightYellow),
            StepStatus::Failed(_) => ("✗", Color::Red),
        }
    }
}

impl LoadingState {
    pub fn new() -> Self {
        Self {
            steps: vec![
                ("Loading configuration...".to_string(), StepStatus::Pending),
                ("Initializing web server...".to_string(), StepStatus::Pending),
                ("Verifying SSH connections...".to_string(), StepStatus::Pending),
                ("Initializing certificate manager...".to_string(), StepStatus::Pending),
            ],
            current_step: 0,
        }
    }

    pub fn next_step(&mut self) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].1 = StepStatus::Complete;
            self.current_step += 1;
            if self.current_step < self.steps.len() {
                self.steps[self.current_step].1 = StepStatus::InProgress;
            }
        }
    }
}


pub use onboarding::OnboardingState;
pub use render::render_all;
pub use styles::*;

