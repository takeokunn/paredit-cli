#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallGraphPolicyOptions {
    fail_on_inbound_callers: bool,
    require_edges: Option<usize>,
    require_internal_edges: Option<usize>,
}

impl CallGraphPolicyOptions {
    pub const fn new(
        fail_on_inbound_callers: bool,
        require_edges: Option<usize>,
        require_internal_edges: Option<usize>,
    ) -> Self {
        Self {
            fail_on_inbound_callers,
            require_edges,
            require_internal_edges,
        }
    }

    pub const fn fail_on_inbound_callers(&self) -> bool {
        self.fail_on_inbound_callers
    }

    pub const fn require_edges(&self) -> Option<usize> {
        self.require_edges
    }

    pub const fn require_internal_edges(&self) -> Option<usize> {
        self.require_internal_edges
    }
}
