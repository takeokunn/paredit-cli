use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityOverlapPolicy {
    Maximal,
    All,
}

impl SimilarityOverlapPolicy {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Maximal => "maximal",
            Self::All => "all",
        }
    }
}

impl FromStr for SimilarityOverlapPolicy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "maximal" => Ok(Self::Maximal),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown overlap policy: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityComparisonScope {
    All,
    SameFile,
    CrossFile,
}

impl SimilarityComparisonScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::SameFile => "same-file",
            Self::CrossFile => "cross-file",
        }
    }
}

impl FromStr for SimilarityComparisonScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "same-file" => Ok(Self::SameFile),
            "cross-file" => Ok(Self::CrossFile),
            _ => Err(format!("unknown comparison scope: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityFormScope {
    All,
    TopLevel,
}

impl SimilarityFormScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::TopLevel => "top-level",
        }
    }
}

impl FromStr for SimilarityFormScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "top-level" => Ok(Self::TopLevel),
            _ => Err(format!("unknown form scope: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarityReportOptions {
    threshold: f64,
    min_node_count: usize,
    min_line_span: usize,
    comparison_scope: SimilarityComparisonScope,
    form_scope: SimilarityFormScope,
    overlap_policy: SimilarityOverlapPolicy,
    max_candidates: Option<usize>,
    max_comparisons: Option<usize>,
    max_results: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarityReportOptionsBuilder {
    options: SimilarityReportOptions,
}

impl SimilarityReportOptions {
    pub fn builder() -> SimilarityReportOptionsBuilder {
        SimilarityReportOptionsBuilder {
            options: Self::default(),
        }
    }

    fn validate(self) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&self.threshold) {
            return Err("--threshold must be between 0.0 and 1.0".to_string());
        }
        if self.min_node_count < 2 {
            return Err("--min-node-count must be at least 2".to_string());
        }
        if self.min_line_span == 0 {
            return Err("--min-line-span must be at least 1".to_string());
        }
        if self.max_candidates == Some(0) {
            return Err("--max-candidates must be at least 1".to_string());
        }
        if self.max_comparisons == Some(0) {
            return Err("--max-comparisons must be at least 1".to_string());
        }
        if self.max_results == Some(0) {
            return Err("--max-results must be at least 1".to_string());
        }

        Ok(self)
    }

    pub const fn threshold(self) -> f64 {
        self.threshold
    }

    pub const fn min_node_count(self) -> usize {
        self.min_node_count
    }

    pub const fn min_line_span(self) -> usize {
        self.min_line_span
    }

    pub const fn comparison_scope(self) -> SimilarityComparisonScope {
        self.comparison_scope
    }

    pub const fn form_scope(self) -> SimilarityFormScope {
        self.form_scope
    }

    pub const fn overlap_policy(self) -> SimilarityOverlapPolicy {
        self.overlap_policy
    }

    pub const fn max_candidates(self) -> Option<usize> {
        self.max_candidates
    }

    pub const fn max_comparisons(self) -> Option<usize> {
        self.max_comparisons
    }

    pub const fn max_results(self) -> Option<usize> {
        self.max_results
    }
}

impl SimilarityReportOptionsBuilder {
    pub const fn threshold(mut self, threshold: f64) -> Self {
        self.options.threshold = threshold;
        self
    }

    pub const fn min_node_count(mut self, min_node_count: usize) -> Self {
        self.options.min_node_count = min_node_count;
        self
    }

    pub const fn min_line_span(mut self, min_line_span: usize) -> Self {
        self.options.min_line_span = min_line_span;
        self
    }

    pub const fn comparison_scope(mut self, comparison_scope: SimilarityComparisonScope) -> Self {
        self.options.comparison_scope = comparison_scope;
        self
    }

    pub const fn form_scope(mut self, form_scope: SimilarityFormScope) -> Self {
        self.options.form_scope = form_scope;
        self
    }

    pub const fn overlap_policy(mut self, overlap_policy: SimilarityOverlapPolicy) -> Self {
        self.options.overlap_policy = overlap_policy;
        self
    }

    pub const fn max_candidates(mut self, max_candidates: Option<usize>) -> Self {
        self.options.max_candidates = max_candidates;
        self
    }

    pub const fn max_comparisons(mut self, max_comparisons: Option<usize>) -> Self {
        self.options.max_comparisons = max_comparisons;
        self
    }

    pub const fn max_results(mut self, max_results: Option<usize>) -> Self {
        self.options.max_results = max_results;
        self
    }

    pub fn build(self) -> Result<SimilarityReportOptions, String> {
        self.options.validate()
    }
}

impl Default for SimilarityReportOptions {
    fn default() -> Self {
        Self {
            threshold: 0.87,
            min_node_count: 4,
            min_line_span: 1,
            comparison_scope: SimilarityComparisonScope::All,
            form_scope: SimilarityFormScope::All,
            overlap_policy: SimilarityOverlapPolicy::Maximal,
            max_candidates: None,
            max_comparisons: None,
            max_results: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_stable() {
        assert_eq!(SimilarityComparisonScope::All.label(), "all");
        assert_eq!(SimilarityComparisonScope::SameFile.label(), "same-file");
        assert_eq!(SimilarityComparisonScope::CrossFile.label(), "cross-file");
        assert_eq!(SimilarityFormScope::All.label(), "all");
        assert_eq!(SimilarityFormScope::TopLevel.label(), "top-level");
        assert_eq!(SimilarityOverlapPolicy::Maximal.label(), "maximal");
        assert_eq!(SimilarityOverlapPolicy::All.label(), "all");
    }

    #[test]
    fn parses_expected_labels() {
        assert!(matches!(
            "all".parse::<SimilarityComparisonScope>(),
            Ok(SimilarityComparisonScope::All)
        ));
        assert!(matches!(
            "same-file".parse::<SimilarityComparisonScope>(),
            Ok(SimilarityComparisonScope::SameFile)
        ));
        assert!(matches!(
            "cross-file".parse::<SimilarityComparisonScope>(),
            Ok(SimilarityComparisonScope::CrossFile)
        ));
        assert!(matches!(
            "all".parse::<SimilarityFormScope>(),
            Ok(SimilarityFormScope::All)
        ));
        assert!(matches!(
            "top-level".parse::<SimilarityFormScope>(),
            Ok(SimilarityFormScope::TopLevel)
        ));
        assert!(matches!(
            "maximal".parse::<SimilarityOverlapPolicy>(),
            Ok(SimilarityOverlapPolicy::Maximal)
        ));
        assert!(matches!(
            "all".parse::<SimilarityOverlapPolicy>(),
            Ok(SimilarityOverlapPolicy::All)
        ));
    }
}
