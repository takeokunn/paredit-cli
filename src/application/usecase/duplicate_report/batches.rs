use std::collections::BTreeMap;
use std::path::PathBuf;

use super::types::{DuplicateCandidateGroups, DuplicateFormReport, ReplacementPlanBatch};

pub fn collect_replacement_plan_batches(
    grouped: DuplicateCandidateGroups,
    min_group_size: usize,
    replacement: String,
    keep_first: bool,
) -> Vec<ReplacementPlanBatch> {
    let mut batches = Vec::new();

    for (shape, forms) in grouped {
        let mut by_file = BTreeMap::<PathBuf, Vec<DuplicateFormReport>>::new();
        for form in forms {
            by_file.entry(form.path.clone()).or_default().push(form);
        }

        for (file, mut file_forms) in by_file {
            if file_forms.len() < min_group_size {
                continue;
            }

            file_forms.sort_by_key(|form| form.span.start().get());
            let Some(first_form) = file_forms.first() else {
                continue;
            };

            batches.push(ReplacementPlanBatch {
                file,
                dialect: first_form.dialect,
                shape: shape.clone(),
                replacement: replacement.clone(),
                keep_first,
                forms: file_forms,
            });
        }
    }

    batches
}
