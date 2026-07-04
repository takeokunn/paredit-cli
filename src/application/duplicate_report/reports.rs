use super::types::{DuplicateCandidateGroups, DuplicateShapeReport};

pub fn build_duplicate_shape_reports(
    grouped: DuplicateCandidateGroups,
    min_group_size: usize,
) -> Vec<DuplicateShapeReport> {
    let mut reports = grouped
        .into_iter()
        .filter_map(|(shape, forms)| {
            (forms.len() >= min_group_size).then_some(DuplicateShapeReport {
                count: forms.len(),
                shape,
                forms,
            })
        })
        .collect::<Vec<_>>();

    reports.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.shape.cmp(&right.shape))
    });

    reports
}
