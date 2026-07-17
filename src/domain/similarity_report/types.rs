use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::dialect::Dialect;
use crate::domain::form_similarity::StructuralTree;
use crate::domain::sexpr::{ByteOffset, ByteSpan, Path};

#[allow(unused_imports)]
pub use crate::domain::similarity_report::{
    SimilarityComparisonScope, SimilarityFormScope, SimilarityOverlapPolicy,
    SimilarityReportOptions, SimilarityReportOptionsError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedFormText {
    source: Arc<str>,
    span: ByteSpan,
}

impl SharedFormText {
    fn owned(text: String) -> Self {
        let len = text.len();
        Self {
            source: Arc::from(text),
            span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(len)),
        }
    }

    pub(crate) fn from_source(source: Arc<str>, span: ByteSpan) -> Self {
        Self { source, span }
    }

    pub(crate) fn source_identity(&self) -> *const str {
        Arc::as_ptr(&self.source)
    }

    pub(crate) fn source_len(&self) -> usize {
        self.source.len()
    }

    #[cfg(test)]
    pub(crate) fn shares_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
    }
}

impl Deref for SharedFormText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.span.slice(&self.source)
    }
}

impl AsRef<str> for SharedFormText {
    fn as_ref(&self) -> &str {
        self
    }
}

impl PartialEq<str> for SharedFormText {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<&str> for SharedFormText {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityFormReport {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub form_path: Path,
    pub span: ByteSpan,
    pub node_count: usize,
    pub head: Option<FormHead>,
    pub text: SharedFormText,
}

impl SimilarityFormReport {
    pub fn new(
        path: PathBuf,
        dialect: Dialect,
        form_path: Path,
        span: ByteSpan,
        node_count: usize,
        head: Option<FormHead>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            path,
            dialect,
            form_path,
            span,
            node_count,
            head,
            text: SharedFormText::owned(text.into()),
        }
    }

    pub(crate) fn new_shared(
        path: PathBuf,
        dialect: Dialect,
        form_path: Path,
        span: ByteSpan,
        node_count: usize,
        head: Option<FormHead>,
        source: Arc<str>,
    ) -> Self {
        Self {
            path,
            dialect,
            form_path,
            span,
            node_count,
            head,
            text: SharedFormText::from_source(source, span),
        }
    }

    pub fn contains_span(&self, other: &Self) -> bool {
        self.span.contains_span(other.span)
    }

    pub fn strictly_contains_span(&self, other: &Self) -> bool {
        self.contains_span(other) && self.span != other.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityCandidate {
    pub form: Arc<SimilarityFormReport>,
    pub tree: StructuralTree,
    pub comparison_head: Option<ComparisonHead>,
}

impl SimilarityCandidate {
    pub fn new(
        form: impl Into<Arc<SimilarityFormReport>>,
        tree: StructuralTree,
        comparison_head: Option<ComparisonHead>,
    ) -> Self {
        Self {
            form: form.into(),
            tree,
            comparison_head,
        }
    }

    pub fn same_comparison_bucket(&self, other: &Self) -> bool {
        self.comparison_head == other.comparison_head
    }

    pub fn cmp_comparison_bucket(&self, other: &Self) -> Ordering {
        self.comparison_head.cmp(&other.comparison_head)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityPairReport {
    pub similarity: f64,
    pub score: f64,
    pub left: Arc<SimilarityFormReport>,
    pub right: Arc<SimilarityFormReport>,
}

impl SimilarityPairReport {
    pub fn new(
        similarity: f64,
        score: f64,
        left: SimilarityFormReport,
        right: SimilarityFormReport,
    ) -> Self {
        Self {
            similarity,
            score,
            left: Arc::new(left),
            right: Arc::new(right),
        }
    }

    pub(crate) fn from_shared(
        similarity: f64,
        score: f64,
        left: Arc<SimilarityFormReport>,
        right: Arc<SimilarityFormReport>,
    ) -> Self {
        Self {
            similarity,
            score,
            left,
            right,
        }
    }

    pub fn strictly_contains_pair(&self, other: &Self) -> bool {
        strictly_contains_pair_forms(&self.left, &self.right, &other.left, &other.right)
    }
}

pub(super) fn strictly_contains_pair_forms(
    left_outer: &SimilarityFormReport,
    right_outer: &SimilarityFormReport,
    left_inner: &SimilarityFormReport,
    right_inner: &SimilarityFormReport,
) -> bool {
    left_outer.strictly_contains_span(left_inner) && right_outer.contains_span(right_inner)
        || left_outer.contains_span(left_inner) && right_outer.strictly_contains_span(right_inner)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComparisonHead(String);

impl ComparisonHead {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ComparisonHead {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ComparisonHead {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl From<ComparisonHead> for String {
    fn from(value: ComparisonHead) -> Self {
        value.0
    }
}

impl AsRef<str> for ComparisonHead {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormHead(String);

impl FormHead {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FormHead {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for FormHead {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl From<FormHead> for String {
    fn from(value: FormHead) -> Self {
        value.0
    }
}

impl AsRef<str> for FormHead {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for FormHead {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for FormHead {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityReportSummary {
    pub candidate_limit_reached: bool,
    pub omitted_candidates: usize,
    pub possible_pairs: usize,
    pub evaluated_pairs: usize,
    pub pruned_by_size: usize,
    pub resource_skipped_pairs: usize,
    pub comparison_limit_reached: bool,
    pub unprocessed_pairs: usize,
    pub matched_pairs: usize,
    pub suppressed_pairs: usize,
    pub reported_pairs: usize,
    pub truncated: bool,
}

impl SimilarityReportSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        candidate_limit_reached: bool,
        omitted_candidates: usize,
        possible_pairs: usize,
        evaluated_pairs: usize,
        pruned_by_size: usize,
        resource_skipped_pairs: usize,
        comparison_limit_reached: bool,
        unprocessed_pairs: usize,
        matched_pairs: usize,
        suppressed_pairs: usize,
        reported_pairs: usize,
        truncated: bool,
    ) -> Self {
        Self {
            candidate_limit_reached,
            omitted_candidates,
            possible_pairs,
            evaluated_pairs,
            pruned_by_size,
            resource_skipped_pairs,
            comparison_limit_reached,
            unprocessed_pairs,
            matched_pairs,
            suppressed_pairs,
            reported_pairs,
            truncated,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityReport {
    pub summary: SimilarityReportSummary,
    pub pairs: Vec<SimilarityPairReport>,
}

impl SimilarityReport {
    pub fn new(summary: SimilarityReportSummary, pairs: Vec<SimilarityPairReport>) -> Self {
        Self { summary, pairs }
    }
}
