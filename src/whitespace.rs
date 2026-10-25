use serde::Deserialize;
use unicode_width::UnicodeWidthStr;

use crate::diff::{CellKind, DiffCell};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhitespaceMode {
    Off,
    #[default]
    Changed,
    All,
}

impl WhitespaceMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Changed, Self::All];

    pub fn shows(self, kind: &CellKind) -> bool {
        match self {
            Self::Off => false,
            Self::Changed => matches!(kind, CellKind::Delete | CellKind::Insert),
            Self::All => *kind != CellKind::Empty,
        }
    }
}

impl std::fmt::Display for WhitespaceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Off => "Whitespace: Off",
            Self::Changed => "Whitespace: Changed",
            Self::All => "Whitespace: All",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Space,
    Tab,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplaySpan {
    pub text: String,
    pub emphasized: bool,
    pub marker: Option<Marker>,
}

/// Prepare layout text without inserting marker glyphs or mutating source spans.
/// Tab expansion is identical in every visibility mode, including Off.
pub fn display_spans(cell: &DiffCell, tab_width: usize) -> Vec<DisplaySpan> {
    let tab_width = tab_width.clamp(1, 16);
    let mut output = Vec::new();
    let mut column = 0;
    let mut since_tab = String::new();
    for span in &cell.spans {
        let mut word = String::new();
        for ch in span.text.chars() {
            if ch == ' ' || ch == '\t' {
                if !word.is_empty() {
                    output.push(DisplaySpan {
                        text: std::mem::take(&mut word),
                        emphasized: span.emphasized,
                        marker: None,
                    });
                }
                let (text, marker) = if ch == '\t' {
                    column += since_tab.width();
                    since_tab.clear();
                    let width = tab_width - column % tab_width;
                    column += width;
                    (" ".repeat(width), Marker::Tab)
                } else {
                    since_tab.push(ch);
                    (" ".to_string(), Marker::Space)
                };
                output.push(DisplaySpan {
                    text,
                    emphasized: span.emphasized,
                    marker: Some(marker),
                });
            } else {
                since_tab.push(ch);
                word.push(ch);
            }
        }
        if !word.is_empty() {
            output.push(DisplaySpan {
                text: word,
                emphasized: span.emphasized,
                marker: None,
            });
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::{compute_diff, DiffSpan};

    fn cell(parts: &[(&str, bool)]) -> DiffCell {
        DiffCell {
            line_number: Some(1),
            kind: CellKind::Insert,
            spans: parts
                .iter()
                .map(|(text, emphasized)| DiffSpan {
                    text: text.to_string(),
                    emphasized: *emphasized,
                })
                .collect(),
            newline_marker: None,
        }
    }

    #[test]
    fn spaces_tabs_and_trailing_whitespace_remain_distinguishable() {
        let original = cell(&[("a \tb  \t", false)]);
        let spans = display_spans(&original, 4);
        assert_eq!(
            spans
                .iter()
                .map(|span| span.text.as_str())
                .collect::<String>(),
            "a   b   "
        );
        assert_eq!(
            spans
                .iter()
                .filter(|span| span.marker == Some(Marker::Space))
                .count(),
            3
        );
        let tabs: Vec<_> = spans
            .iter()
            .filter(|span| span.marker == Some(Marker::Tab))
            .map(|span| span.text.len())
            .collect();
        assert_eq!(tabs, vec![2, 1]);
        assert_eq!(original.spans[0].text, "a \tb  \t");
    }

    #[test]
    fn tab_stops_cross_diff_spans_and_respect_unicode_width() {
        let spans = display_spans(
            &cell(&[("界", false), ("e\u{301}", true), ("\tx\t", true)]),
            4,
        );
        let tabs: Vec<_> = spans
            .iter()
            .filter(|span| span.marker == Some(Marker::Tab))
            .map(|span| (span.text.len(), span.emphasized))
            .collect();
        assert_eq!(tabs, vec![(1, true), (3, true)]);
        assert_eq!(
            display_spans(&cell(&[("\t\t", false)]), 8)
                .iter()
                .map(|span| span.text.len())
                .collect::<Vec<_>>(),
            vec![8, 8]
        );
    }

    #[test]
    fn tab_width_is_bounded_and_empty_cells_have_no_markers() {
        let tab = cell(&[("\t", false)]);
        assert_eq!(display_spans(&tab, 0)[0].text.len(), 1);
        assert_eq!(display_spans(&tab, usize::MAX)[0].text.len(), 16);
        let diff = compute_diff("", "new\n");
        assert!(display_spans(&diff.rows[0].left, 4).is_empty());
    }

    #[test]
    fn visibility_modes_select_only_real_lines() {
        for kind in [
            CellKind::Equal,
            CellKind::Delete,
            CellKind::Insert,
            CellKind::Empty,
        ] {
            assert!(!WhitespaceMode::Off.shows(&kind));
            assert_eq!(
                WhitespaceMode::Changed.shows(&kind),
                matches!(kind, CellKind::Delete | CellKind::Insert)
            );
            assert_eq!(WhitespaceMode::All.shows(&kind), kind != CellKind::Empty);
        }
    }
}
