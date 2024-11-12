use super::ClusterConfig;
use glob::glob;

#[derive(Clone)]
pub struct ConfigEditor {
    pub fields: Vec<String>,
    pub current_field: usize,
    pub editing_value: String,
    pub is_editing: bool,
    pub completions: Vec<String>,
    pub selected_completion: usize,
}

impl ConfigEditor {
    pub fn new(config: &ClusterConfig) -> Self {
        ConfigEditor {
            fields: vec![
                config.remote_user.clone(),
                config.control_plane.clone(),
                config.worker_nodes.join(","),
                config.remote_dir.clone(),
                config.ssh_key_path.clone(),
            ],
            current_field: 0,
            editing_value: String::new(),
            is_editing: false,
            completions: Vec::new(),
            selected_completion: 0,
        }
    }

    pub fn apply_to_config(&self, config: &mut ClusterConfig) {
        config.remote_user = self.fields[0].clone();
        config.control_plane = self.fields[1].clone();
        config.worker_nodes = self.fields[2]
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        config.remote_dir = self.fields[3].clone();
        config.ssh_key_path = self.fields[4].clone();
    }

    pub fn handle_tab(&mut self) {
        if self.current_field == 4 {
            // SSH key path field
            if self.completions.is_empty() {
                self.completions = self.get_path_completions(&self.editing_value);
                self.selected_completion = 0;
            } else {
                self.selected_completion = (self.selected_completion + 1) % self.completions.len();
            }

            if !self.completions.is_empty() {
                self.editing_value = self.completions[self.selected_completion].clone();
            }
        }
    }

    pub fn reset_completions(&mut self) {
        self.completions.clear();
        self.selected_completion = 0;
    }

    fn get_path_completions(&self, partial_path: &str) -> Vec<String> {
        let mut completions = Vec::new();
        let path = shellexpand::tilde(partial_path).to_string();
        let pattern = if path.ends_with('/') {
            format!("{}*", path)
        } else {
            format!("{}*", path)
        };

        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(path_str) = entry.to_str() {
                    let mut completion = path_str.to_string();
                    if entry.is_dir() {
                        completion.push('/');
                    }
                    completions.push(completion);
                }
            }
        }
        completions
    }
}
