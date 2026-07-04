#[derive(Debug, Clone, Copy)]
pub struct RefactorWriteCandidate {
    pub changed: bool,
    pub output_parse_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactorWriteRefusal {
    UnparsableOutputs { count: usize },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteGateInputs {
    pub write_requested: bool,
    pub policy_passed: bool,
    pub outputs_parse: bool,
    pub preflight_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteDecision {
    pub write_parse_refused: bool,
    pub run_pre_verification: bool,
    pub apply_preview: bool,
    pub run_post_verification: bool,
}
