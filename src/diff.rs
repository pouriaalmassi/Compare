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

    let old_lines: Vec<&str> = left_content.lines().collect();
    let new_lines: Vec<&str> = right_content.lines().collect();

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
                            line_number: Some(old_index + i + 1),
                            kind: CellKind::Equal,
                            spans: vec![DiffSpan {
                                text: old_l.to_string(),
                                emphasized: false,
                            }],
                        },
                        right: DiffCell {
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
                            line_number: Some(old_index + i + 1),
                            kind: CellKind::Delete,
                            spans: vec![DiffSpan {
                                text: old_l.to_string(),
                                emphasized: false,
                            }],
                        },
                        right: DiffCell {
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
                            line_number: None,
                            kind: CellKind::Empty,
                            spans: vec![],
                        },
                        right: DiffCell {
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
                                line_number: Some(old_index + i + 1),
                                kind: CellKind::Delete,
                                spans: old_spans,
                            },
                            right: DiffCell {
                                line_number: Some(new_index + i + 1),
                                kind: CellKind::Insert,
                                spans: new_spans,
                            },
                        });
                    } else if has_old {
                        let old_l = old_lines.get(old_index + i).copied().unwrap_or("");
                        rows.push(DiffRow {
                            left: DiffCell {
                                line_number: Some(old_index + i + 1),
                                kind: CellKind::Delete,
                                spans: vec![DiffSpan {
                                    text: old_l.to_string(),
                                    emphasized: false,
                                }],
                            },
                            right: DiffCell {
                                line_number: None,
                                kind: CellKind::Empty,
                                spans: vec![],
                            },
                        });
                    } else {
                        let new_l = new_lines.get(new_index + i).copied().unwrap_or("");
                        rows.push(DiffRow {
                            left: DiffCell {
                                line_number: None,
                                kind: CellKind::Empty,
                                spans: vec![],
                            },
                            right: DiffCell {
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

    DiffData { rows, differ_count }
}

#[cfg(test)]
mod tests {
    use super::*;

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
