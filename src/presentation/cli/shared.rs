mod diff;
mod input;
mod write;

pub(super) use diff::{apply_byte_span_edits, bounded_preview, stable_text_hash, unified_diff};
pub(super) use input::{
    detect_dialect, edit_target, expand_input_paths, list_head, matching_symbol_occurrences,
    package_context_before_top_level, read_file_or_empty, read_input, require_output_file,
    resolve_target,
};
pub(super) use write::{write_file_with_rollback, write_files_with_rollback};
