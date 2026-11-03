use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestArchitecture {
    Arm32,
    Arm64,
    X86,
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    System,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmulatorConfig {
    pub architecture: GuestArchitecture,
    pub mode: ExecutionMode,
    pub memory_mib: u32,
    pub vcpus: u8,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            architecture: GuestArchitecture::X86,
            mode: ExecutionMode::System,
            memory_mib: 512,
            vcpus: 1,
        }
    }
}

impl EmulatorConfig {
    pub fn validate(&self) -> Result<(), EmulatorError> {
        if !(64..=65_536).contains(&self.memory_mib) {
            return Err(EmulatorError::InvalidConfiguration(
                "memory_mib must be between 64 and 65536".to_owned(),
            ));
        }

        if !(1..=32).contains(&self.vcpus) {
            return Err(EmulatorError::InvalidConfiguration(
                "vcpus must be between 1 and 32".to_owned(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
}

impl fmt::Display for EmulatorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Created => "created",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorAction {
    Start,
    Pause,
    Resume,
    Stop,
}

impl fmt::Display for EmulatorAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Start => "start",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Stop => "stop",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    message: String,
}

impl BackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BackendError {}

pub trait EmulationBackend {
    fn name(&self) -> &str;
    fn supports(&self, config: &EmulatorConfig) -> bool;
    fn start(&mut self, config: &EmulatorConfig) -> Result<(), BackendError>;
    fn pause(&mut self) -> Result<(), BackendError>;
    fn resume(&mut self) -> Result<(), BackendError>;
    fn shutdown(&mut self) -> Result<(), BackendError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmulatorError {
    InvalidConfiguration(String),
    UnsupportedConfiguration {
        backend: String,
        architecture: GuestArchitecture,
        mode: ExecutionMode,
    },
    InvalidState {
        action: EmulatorAction,
        state: EmulatorState,
    },
    Backend {
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(message) => write!(f, "Invalid configuration: {message}"),
            Self::UnsupportedConfiguration {
                backend,
                architecture,
                mode,
            } => write!(
                f,
                "Backend {backend} does not support {architecture:?} in {mode:?} mode"
            ),
            Self::InvalidState { action, state } => {
                write!(f, "Cannot {action} emulator while it is {state}")
            }
            Self::Backend { operation, message } => {
                write!(f, "Backend failed during {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for EmulatorError {}

pub struct Emulator<B: EmulationBackend> {
    config: EmulatorConfig,
    backend: B,
    state: EmulatorState,
}

impl<B: EmulationBackend> Emulator<B> {
    pub fn new(config: EmulatorConfig, backend: B) -> Result<Self, EmulatorError> {
        config.validate()?;

        if !backend.supports(&config) {
            return Err(EmulatorError::UnsupportedConfiguration {
                backend: backend.name().to_owned(),
                architecture: config.architecture,
                mode: config.mode,
            });
        }

        Ok(Self {
            config,
            backend,
            state: EmulatorState::Created,
        })
    }

    pub fn config(&self) -> &EmulatorConfig {
        &self.config
    }

    pub fn state(&self) -> EmulatorState {
        self.state
    }

    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }

    pub fn start(&mut self) -> Result<(), EmulatorError> {
        self.require_state(
            EmulatorAction::Start,
            &[EmulatorState::Created, EmulatorState::Stopped],
        )?;
        self.state = EmulatorState::Starting;

        match self.backend.start(&self.config) {
            Ok(()) => {
                self.state = EmulatorState::Running;
                Ok(())
            }
            Err(error) => {
                self.state = EmulatorState::Failed;
                Err(EmulatorError::Backend {
                    operation: "start",
                    message: error.to_string(),
                })
            }
        }
    }

    pub fn pause(&mut self) -> Result<(), EmulatorError> {
        self.require_state(EmulatorAction::Pause, &[EmulatorState::Running])?;

        match self.backend.pause() {
            Ok(()) => {
                self.state = EmulatorState::Paused;
                Ok(())
            }
            Err(error) => {
                self.state = EmulatorState::Failed;
                Err(EmulatorError::Backend {
                    operation: "pause",
                    message: error.to_string(),
                })
            }
        }
    }

    pub fn resume(&mut self) -> Result<(), EmulatorError> {
        self.require_state(EmulatorAction::Resume, &[EmulatorState::Paused])?;

        match self.backend.resume() {
            Ok(()) => {
                self.state = EmulatorState::Running;
                Ok(())
            }
            Err(error) => {
                self.state = EmulatorState::Failed;
                Err(EmulatorError::Backend {
                    operation: "resume",
                    message: error.to_string(),
                })
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), EmulatorError> {
        if self.state == EmulatorState::Stopped {
            return Ok(());
        }

        if self.state == EmulatorState::Created {
            self.state = EmulatorState::Stopped;
            return Ok(());
        }

        self.require_state(
            EmulatorAction::Stop,
            &[
                EmulatorState::Running,
                EmulatorState::Paused,
                EmulatorState::Failed,
            ],
        )?;
        self.state = EmulatorState::Stopping;

        match self.backend.shutdown() {
            Ok(()) => {
                self.state = EmulatorState::Stopped;
                Ok(())
            }
            Err(error) => {
                self.state = EmulatorState::Failed;
                Err(EmulatorError::Backend {
                    operation: "stop",
                    message: error.to_string(),
                })
            }
        }
    }

    fn require_state(
        &self,
        action: EmulatorAction,
        allowed: &[EmulatorState],
    ) -> Result<(), EmulatorError> {
        if allowed.contains(&self.state) {
            Ok(())
        } else {
            Err(EmulatorError::InvalidState {
                action,
                state: self.state,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBackend;

    impl EmulationBackend for TestBackend {
        fn name(&self) -> &str {
            "test-backend"
        }

        fn supports(&self, _config: &EmulatorConfig) -> bool {
            true
        }

        fn start(&mut self, _config: &EmulatorConfig) -> Result<(), BackendError> {
            Ok(())
        }

        fn pause(&mut self) -> Result<(), BackendError> {
            Ok(())
        }

        fn resume(&mut self) -> Result<(), BackendError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), BackendError> {
            Ok(())
        }
    }

    #[test]
    fn lifecycle_moves_through_running_paused_and_stopped() {
        let mut emulator = Emulator::new(EmulatorConfig::default(), TestBackend).unwrap();

        assert_eq!(emulator.state(), EmulatorState::Created);
        emulator.start().unwrap();
        assert_eq!(emulator.state(), EmulatorState::Running);
        emulator.pause().unwrap();
        assert_eq!(emulator.state(), EmulatorState::Paused);
        emulator.resume().unwrap();
        assert_eq!(emulator.state(), EmulatorState::Running);
        emulator.stop().unwrap();
        assert_eq!(emulator.state(), EmulatorState::Stopped);
    }

    #[test]
    fn invalid_transition_returns_error() {
        let mut emulator = Emulator::new(EmulatorConfig::default(), TestBackend).unwrap();

        assert!(matches!(
            emulator.pause(),
            Err(EmulatorError::InvalidState {
                action: EmulatorAction::Pause,
                state: EmulatorState::Created
            })
        ));
    }

    #[test]
    fn rejects_invalid_memory_configuration() {
        let config = EmulatorConfig {
            memory_mib: 32,
            ..EmulatorConfig::default()
        };

        assert!(matches!(
            Emulator::new(config, TestBackend),
            Err(EmulatorError::InvalidConfiguration(_))
        ));
    }
}
