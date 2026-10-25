use similar::{ChangeTag, DiffOp, TextDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellKind {
    Equal,
    Delete,
    Insert,
    Empty,
}

#[derive(Debug, Clone)]
pub struct DiffSpan {
    pub text: String,
    pub emphasized: bool,
}

#[derive(Debug, Clone)]
pub struct DiffCell {
    pub line_number: Option<usize>,
    pub kind: CellKind,
    pub spans: Vec<DiffSpan>,
    pub newline_marker: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct DiffRow {
    pub left: DiffCell,
    pub right: DiffCell,
}

#[derive(Debug, Clone, Default)]
pub struct DiffData {
    pub rows: Vec<DiffRow>,
    pub differ_count: usize,
}

impl DiffData {
    /// The first row of each contiguous block of changed lines.
    pub fn change_starts(&self) -> Vec<usize> {
        let mut was_changed = false;
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                let changed = row.left.kind != CellKind::Equal || row.right.kind != CellKind::Equal;
                let start = changed && !was_changed;
                was_changed = changed;
                start.then_some(index)
            })
            .collect()
    }
}

fn line_text(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line)
}

fn line_ending(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "CRLF"
    } else if line.ends_with('\n') {
        "LF"
    } else if line.ends_with('\r') {
        "CR"
    } else {
        "No final newline"
    }
}

pub fn append_span(spans: &mut Vec<DiffSpan>, text: &str, emphasized: bool) {
    if let Some(last) = spans.last_mut() {
        if last.emphasized == emphasized {
            last.text.push_str(text);
            return;
        }
    }
    spans.push(DiffSpan {
        text: text.to_string(),
        emphasized,
    });
}

pub fn char_diff_lines(old_line: &str, new_line: &str) -> (Vec<DiffSpan>, Vec<DiffSpan>) {
    let char_diff = TextDiff::from_chars(old_line, new_line);
    let mut old_spans: Vec<DiffSpan> = Vec::new();
    let mut new_spans: Vec<DiffSpan> = Vec::new();

    for change in char_diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                append_span(&mut old_spans, change.value(), false);
                append_span(&mut new_spans, change.value(), false);
            }
            ChangeTag::Delete => {
                append_span(&mut old_spans, change.value(), true);
            }
            ChangeTag::Insert => {
                append_span(&mut new_spans, change.value(), true);
            }
        }
    }

    (old_spans, new_spans)
}

pub fn compute_diff(left_content: &str, right_content: &str) -> DiffData {
    let diff = TextDiff::from_lines(left_content, right_content);
    let mut rows = Vec::new();
    let mut differ_count = 0;

    let old_raw: Vec<&str> = diff.iter_old_slices().collect();
    let new_raw: Vec<&str> = diff.iter_new_slices().collect();
    let old_lines: Vec<&str> = old_raw.iter().map(|line| line_text(line)).collect();
    let new_lines: Vec<&str> = new_raw.iter().map(|line| line_text(line)).collect();

    for op in diff.ops() {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for i in 0..*len {
                    let old_l = old_lines.get(old_index + i).copied().unwrap_or("");
                    let new_l = new_lines.get(new_index + i).copied().unwrap_or("");

                    rows.push(DiffRow {
                        left: DiffCell {
                            newline_marker: None,
                            line_number: Some(old_index + i + 1),
                            kind: CellKind::Equal,
                            spans: vec![DiffSpan {
                                text: old_l.to_string(),
                                emphasized: false,
                            }],
                        },
                        right: DiffCell {
                            newline_marker: None,
                            line_number: Some(new_index + i + 1),
                            kind: CellKind::Equal,
                            spans: vec![DiffSpan {
                                text: new_l.to_string(),
                                emphasized: false,
                            }],
                        },
                    });
                }
            }
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                differ_count += *old_len;
                for i in 0..*old_len {
                    let old_l = old_lines.get(old_index + i).copied().unwrap_or("");
                    rows.push(DiffRow {
                        left: DiffCell {
                            newline_marker: None,
                            line_number: Some(old_index + i + 1),
                            kind: CellKind::Delete,
                            spans: vec![DiffSpan {
                                text: old_l.to_string(),
                                emphasized: false,
                            }],
                        },
                        right: DiffCell {
                            newline_marker: None,
                            line_number: None,
                            kind: CellKind::Empty,
                            spans: vec![],
                        },
                    });
                }
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                differ_count += *new_len;
                for i in 0..*new_len {
                    let new_l = new_lines.get(new_index + i).copied().unwrap_or("");
                    rows.push(DiffRow {
                        left: DiffCell {
                            newline_marker: None,
                            line_number: None,
                            kind: CellKind::Empty,
                            spans: vec![],
                        },
                        right: DiffCell {
                            newline_marker: None,
                            line_number: Some(new_index + i + 1),
                            kind: CellKind::Insert,
                            spans: vec![DiffSpan {
                                text: new_l.to_string(),
                                emphasized: false,
                            }],
                        },
                    });
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let max_len = (*old_len).max(*new_len);
                differ_count += max_len;

                for i in 0..max_len {
                    let has_old = i < *old_len;
                    let has_new = i < *new_len;

                    if has_old && has_new {
                        let old_l = old_lines.get(old_index + i).copied().unwrap_or("");
                        let new_l = new_lines.get(new_index + i).copied().unwrap_or("");
                        let (old_spans, new_spans) = char_diff_lines(old_l, new_l);

                        rows.push(DiffRow {
                            left: DiffCell {
                                newline_marker: None,
                                line_number: Some(old_index + i + 1),
                                kind: CellKind::Delete,
                                spans: old_spans,
                            },
                            right: DiffCell {
                                newline_marker: None,
                                line_number: Some(new_index + i + 1),
                                kind: CellKind::Insert,
                                spans: new_spans,
                            },
                        });
                    } else if has_old {
                        let old_l = old_lines.get(old_index + i).copied().unwrap_or("");
                        rows.push(DiffRow {
                            left: DiffCell {
                                newline_marker: None,
                                line_number: Some(old_index + i + 1),
                                kind: CellKind::Delete,
                                spans: vec![DiffSpan {
                                    text: old_l.to_string(),
                                    emphasized: false,
                                }],
                            },
                            right: DiffCell {
                                newline_marker: None,
                                line_number: None,
                                kind: CellKind::Empty,
                                spans: vec![],
                            },
                        });
                    } else {
                        let new_l = new_lines.get(new_index + i).copied().unwrap_or("");
                        rows.push(DiffRow {
                            left: DiffCell {
                                newline_marker: None,
                                line_number: None,
                                kind: CellKind::Empty,
                                spans: vec![],
                            },
                            right: DiffCell {
                                newline_marker: None,
                                line_number: Some(new_index + i + 1),
                                kind: CellKind::Insert,
                                spans: vec![DiffSpan {
                                    text: new_l.to_string(),
                                    emphasized: false,
                                }],
                            },
                        });
                    }
                }
            }
        }
    }

    for row in &mut rows {
        let old = row.left.line_number.map(|n| old_raw[n - 1]);
        let new = row.right.line_number.map(|n| new_raw[n - 1]);
        if row.left.kind != CellKind::Equal || row.right.kind != CellKind::Equal {
            let endings_differ =
                matches!((old, new), (Some(a), Some(b)) if line_ending(a) != line_ending(b));
            for (cell, raw) in [(&mut row.left, old), (&mut row.right, new)] {
                if let Some(raw) = raw {
                    if endings_differ || line_ending(raw) == "No final newline" {
                        cell.newline_marker = Some(line_ending(raw));
                    }
                }
            }
        }
    }
    DiffData { rows, differ_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newline_only_differences_are_visible() {
        for (old, new, left_marker, right_marker) in [
            ("hello\r\n", "hello\n", "CRLF", "LF"),
            ("hello\r", "hello\n", "CR", "LF"),
            ("hello\n", "hello", "LF", "No final newline"),
            ("hello", "hello\r\n", "No final newline", "CRLF"),
            ("\r\n", "\n", "CRLF", "LF"),
        ] {
            let diff = compute_diff(old, new);
            assert_eq!(diff.differ_count, 1);
            assert_eq!(diff.rows.len(), 1);
            assert_eq!(diff.rows[0].left.newline_marker, Some(left_marker));
            assert_eq!(diff.rows[0].right.newline_marker, Some(right_marker));
            for cell in [&diff.rows[0].left, &diff.rows[0].right] {
                assert!(cell
                    .spans
                    .iter()
                    .all(|span| !span.text.contains(['\r', '\n'])));
            }
        }
    }

    #[test]
    fn equal_lines_do_not_show_newline_markers() {
        for content in ["", "same", "same\n", "same\r\n", "\n\n"] {
            let diff = compute_diff(content, content);
            assert_eq!(diff.differ_count, 0);
            assert!(diff.change_starts().is_empty());
            assert!(
                diff.rows
                    .iter()
                    .all(|row| row.left.newline_marker.is_none()
                        && row.right.newline_marker.is_none())
            );
        }
    }

    #[test]
    fn inserted_and_deleted_unterminated_lines_have_markers() {
        let inserted = compute_diff("", "new");
        assert_eq!(
            inserted.rows[0].right.newline_marker,
            Some("No final newline")
        );
        assert!(inserted.rows[0].left.newline_marker.is_none());
        let deleted = compute_diff("old", "");
        assert_eq!(
            deleted.rows[0].left.newline_marker,
            Some("No final newline")
        );
        assert!(deleted.rows[0].right.newline_marker.is_none());
    }

    #[test]
    fn changed_blocks_group_adjacent_rows() {
        let diff = compute_diff("old1\nold2\nkeep\nold3\n", "new1\nnew2\nkeep\nnew3\n");
        assert_eq!(diff.change_starts(), vec![0, 3]);
        assert_eq!(compute_diff("", "one\ntwo\n").change_starts(), vec![0]);
        assert_eq!(compute_diff("one\ntwo\n", "").change_starts(), vec![0]);
    }

    #[test]
    fn test_char_diff_lines() {
        let (old_spans, new_spans) = char_diff_lines("version = \"0.1.0\"", "version = \"0.2.0\"");
        assert_eq!(old_spans.len(), 3);
        assert_eq!(old_spans[0].text, "version = \"0.");
        assert!(!old_spans[0].emphasized);
        assert_eq!(old_spans[1].text, "1");
        assert!(old_spans[1].emphasized);
        assert_eq!(old_spans[2].text, ".0\"");
        assert!(!old_spans[2].emphasized);

        assert_eq!(new_spans.len(), 3);
        assert_eq!(new_spans[0].text, "version = \"0.");
        assert!(!new_spans[0].emphasized);
        assert_eq!(new_spans[1].text, "2");
        assert!(new_spans[1].emphasized);
        assert_eq!(new_spans[2].text, ".0\"");
        assert!(!new_spans[2].emphasized);
    }

    #[test]
    fn test_compute_diff_screenshot_case() {
        let old = "[package]\nname = \"diffitrust\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\niced = \"0.13\"\n\n";
        let new = "[package]\nname = \"diffitrust\"\nname = \"diffitrust\"\nname = \"diffitrust\"\nname = \"diffitrust\"\nname = \"diffitrust\"\nname = \"diffitrust\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\niced = \"0.13\"\niced = \"0.13\"\niced = \"0.13\"\niced = \"0.13\"\niced = \"0.13\"\niced = \"0.13\"\n\n";
        let res = compute_diff(old, new);
        assert_eq!(res.differ_count, 10);
    }
}
