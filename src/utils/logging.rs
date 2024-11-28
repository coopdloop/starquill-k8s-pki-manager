use std::{fs::{self, OpenOptions}, io::Write, path::Path};
use chrono::Local;

pub trait Logger: Send + Sync {
    fn log(&mut self, message: &str);
    fn debug_log(&mut self, message: &str);
}

#[derive(Debug)]
pub struct FileLogger {
    log_file: String,
    debug: bool,
}

impl FileLogger {
    pub fn new(log_file: &str, debug: bool) -> std::io::Result<Self> {
        // Create log directory if it doesn't exist
        if let Some(parent) = Path::new(log_file).parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(FileLogger {
            log_file: log_file.to_string(),
            debug,
        })
    }

    fn write_to_file(&self, message: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        writeln!(file, "{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), message)
    }
}

impl Logger for FileLogger {
    fn log(&mut self, message: &str) {
        if let Err(e) = self.write_to_file(message) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }

    fn debug_log(&mut self, message: &str) {
        if self.debug {
            if let Err(e) = self.write_to_file(&format!("[DEBUG] {}", message)) {
                eprintln!("Failed to write debug log: {}", e);
            }
        }
    }
}

// MultiLogger allows logging to multiple destinations
pub struct MultiLogger {
    loggers: Vec<Box<dyn Logger>>,
}


impl Logger for MultiLogger {
    fn log(&mut self, message: &str) {
        for logger in &mut self.loggers {
            logger.log(message);
        }
    }

    fn debug_log(&mut self, message: &str) {
        for logger in &mut self.loggers {
            logger.debug_log(message);
        }
    }
}

