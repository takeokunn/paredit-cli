#[derive(Debug, Clone, Copy)]
pub struct RefactorWriteCandidate {
    pub changed: bool,
    pub output_parse_ok: bool,
}

pub use crate::domain::refactor_execute::{
    RefactorExecuteDecision, RefactorExecuteDecisionStatus, RefactorExecuteGateInputs,
    RefactorExecuteStep, RefactorExecuteStepStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactorWriteRefusal {
    UnparsableOutputs { count: usize },
}

impl RefactorWriteRefusal {
    pub fn label(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "unparsable-outputs",
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "rewritten-output-did-not-parse",
        }
    }

    pub fn next_action(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "inspect-preview-parse-errors",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactorWritePlan {
    pub write_requested: bool,
    pub writable_indexes: Vec<usize>,
    pub refusal: Option<RefactorWriteRefusal>,
}

impl RefactorWritePlan {
    pub fn write_allowed(&self) -> bool {
        self.write_requested && self.refusal.is_none()
    }
}
