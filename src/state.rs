//! Explicit application state machine.

use crate::error::{Result, TinySttError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Idle,
    LoadingModel,
    Recording,
    Recognizing,
    Completed,
    Cancelled,
    Error,
}

impl AppState {
    pub fn is_idle_like(self) -> bool {
        matches!(self, Self::Idle | Self::Completed | Self::Cancelled)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::LoadingModel => "Loading model...",
            Self::Recording => "Recording...",
            Self::Recognizing => "Recognizing...",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMachine {
    state: AppState,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self {
            state: AppState::LoadingModel,
        }
    }
}

impl StateMachine {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn transition(&mut self, next: AppState) -> Result<()> {
        if self.state == next {
            return Ok(());
        }

        let allowed = match self.state {
            AppState::Idle => matches!(
                next,
                AppState::LoadingModel | AppState::Recording | AppState::Error
            ),
            AppState::LoadingModel => matches!(next, AppState::Idle | AppState::Error),
            AppState::Recording => matches!(
                next,
                AppState::Recognizing | AppState::Cancelled | AppState::Error
            ),
            AppState::Recognizing => {
                matches!(next, AppState::Completed | AppState::Error)
            }
            AppState::Completed => matches!(next, AppState::Idle | AppState::Recording),
            AppState::Cancelled => matches!(next, AppState::Idle | AppState::Recording),
            AppState::Error => matches!(next, AppState::Idle | AppState::LoadingModel),
        };

        if allowed {
            self.state = next;
            Ok(())
        } else {
            Err(TinySttError::InvalidTransition(format!(
                "{:?} -> {:?}",
                self.state, next
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_transitions() {
        let mut sm = StateMachine::new(AppState::Idle);
        sm.transition(AppState::Recording).unwrap();
        sm.transition(AppState::Recognizing).unwrap();
        sm.transition(AppState::Completed).unwrap();
        sm.transition(AppState::Idle).unwrap();
    }

    #[test]
    fn recording_can_be_cancelled() {
        let mut sm = StateMachine::new(AppState::Idle);
        sm.transition(AppState::Recording).unwrap();
        sm.transition(AppState::Cancelled).unwrap();
        assert_eq!(sm.state(), AppState::Cancelled);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut sm = StateMachine::new(AppState::Idle);
        assert!(sm.transition(AppState::Recognizing).is_err());
        assert_eq!(sm.state(), AppState::Idle);
    }
}
