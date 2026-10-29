//! A column for diff rows that visits only visible rows during scrolling.
//! All rows retain their layout and state so wrapping and navigation stay exact.
use std::ops::Range;

use iced::advanced::{
    layout, mouse, renderer,
    widget::{tree, Operation, Tree},
    Clipboard, Layout, Shell, Widget,
};
use iced::{event, Alignment, Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

#[derive(Default)]
struct State {
    bounds: Vec<Rectangle>,
}

pub struct DiffRows<'a> {
    rows: Vec<Element<'a, Message>>,
}

impl<'a> DiffRows<'a> {
    pub fn new(rows: Vec<Element<'a, Message>>) -> Self {
        Self { rows }
    }
}

// Bounds are ordered, non-overlapping, and relative to the column. Binary search
// keeps scrolling work independent of the number of offscreen rows.
fn visible_range(bounds: &[Rectangle], top: f32, bottom: f32) -> Range<usize> {
    let start = bounds.partition_point(|row| row.y + row.height < top);
    let end = bounds.partition_point(|row| row.y <= bottom);
    start..end.max(start)
}

fn visible(state: &State, layout: Layout<'_>, viewport: &Rectangle) -> Range<usize> {
    visible_range(
        &state.bounds,
        viewport.y - layout.position().y,
        viewport.y + viewport.height - layout.position().y,
    )
}

impl Widget<Message, Theme, Renderer> for DiffRows<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.rows.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.rows);
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            limits,
            Length::Fill,
            Length::Shrink,
            iced::Padding::ZERO,
            0.0,
            Alignment::Start,
            &self.rows,
            &mut tree.children,
        );
        tree.state.downcast_mut::<State>().bounds =
            node.children().iter().map(layout::Node::bounds).collect();
        node
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        // Navigation must also be able to find rows outside the viewport.
        operation.container(None, layout.bounds(), &mut |operation| {
            for ((row, state), layout) in self
                .rows
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                row.as_widget().operate(state, layout, renderer, operation);
            }
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let range = visible(tree.state.downcast_ref::<State>(), layout, viewport);
        let layouts = layout.children().skip(range.start).take(range.len());
        self.rows[range.clone()]
            .iter_mut()
            .zip(&mut tree.children[range])
            .zip(layouts)
            .map(|((row, state), layout)| {
                row.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let range = visible(tree.state.downcast_ref::<State>(), layout, viewport);
        let layouts = layout.children().skip(range.start).take(range.len());
        self.rows[range.clone()]
            .iter()
            .zip(&tree.children[range])
            .zip(layouts)
            .map(|((row, state), layout)| {
                row.as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let range = visible(tree.state.downcast_ref::<State>(), layout, viewport);
        let layouts = layout.children().skip(range.start).take(range.len());
        for ((row, state), layout) in self.rows[range.clone()]
            .iter()
            .zip(&tree.children[range])
            .zip(layouts)
        {
            row.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }
}

impl<'a> From<DiffRows<'a>> for Element<'a, Message> {
    fn from(rows: DiffRows<'a>) -> Self {
        Element::new(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(heights: &[f32]) -> Vec<Rectangle> {
        let mut y = 0.0;
        heights
            .iter()
            .map(|&height| {
                let row = Rectangle {
                    x: 0.0,
                    y,
                    width: 500.0,
                    height,
                };
                y += height;
                row
            })
            .collect()
    }

    #[test]
    fn visible_rows_include_wrapped_rows_and_boundary_intersections() {
        let rows = bounds(&[20.0, 100.0, 20.0, 40.0]);
        assert_eq!(visible_range(&rows, 40.0, 80.0), 1..2);
        assert_eq!(visible_range(&rows, 20.0, 120.0), 0..3);
        assert_eq!(visible_range(&rows, -100.0, -1.0), 0..0);
        assert_eq!(visible_range(&rows, 181.0, 200.0), 4..4);
        assert_eq!(visible_range(&[], 0.0, 100.0), 0..0);
    }

    #[test]
    fn viewport_translation_and_large_files_keep_work_bounded() {
        let rows = bounds(&vec![22.2; 30_000]);
        let state = State {
            bounds: rows.clone(),
        };
        let node = layout::Node::new(Size::new(500.0, 700_000.0));
        let layout = Layout::with_offset(iced::Vector::new(0.0, -400_000.0), &node);
        let viewport = Rectangle {
            x: 0.0,
            y: 80.0,
            width: 500.0,
            height: 800.0,
        };
        let range = visible(&state, layout, &viewport);
        assert!(range.start > 18_000);
        assert!(range.len() <= 38);
        // Match a full traversal, including partially visible first/last rows.
        let expected: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.y + row.height >= 400_080.0 && row.y <= 400_880.0)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(range.collect::<Vec<_>>(), expected);
    }

    /// Set COMPARE_PERF_FILE and COMPARE_PERF_OTHER to benchmark a real diff pair.
    /// Run with cargo test --release sample_file -- --ignored --nocapture.
    #[test]
    #[ignore = "requires COMPARE_PERF_FILE and COMPARE_PERF_OTHER"]
    fn sample_file_visible_rows_match_full_traversal() {
        use iced::advanced::text::{self, Paragraph, Renderer as TextRenderer};
        type TextParagraph = <Renderer as TextRenderer>::Paragraph;
        let path = std::env::var("COMPARE_PERF_FILE").expect("set COMPARE_PERF_FILE");
        let source = std::fs::read_to_string(path).unwrap();
        let other_path = std::env::var("COMPARE_PERF_OTHER").expect("set COMPARE_PERF_OTHER");
        let other = std::fs::read_to_string(other_path).unwrap();
        let diff = crate::diff::compute_diff(&source, &other);
        // Shape the sample at a representative pane width, including tabs and
        // wrapped lines, using the same paragraph settings as DiffText.
        let heights: Vec<_> = diff
            .rows
            .iter()
            .map(|row| {
                [&row.left, &row.right]
                    .into_iter()
                    .map(|cell| {
                        let pieces = crate::whitespace::display_spans(cell, 4);
                        let mut spans: Vec<_> = pieces
                            .into_iter()
                            .map(|piece| text::Span::<'static, (), iced::Font>::new(piece.text))
                            .collect();
                        if let Some(marker) = cell.newline_marker {
                            spans.push(text::Span::new(format!("  [{marker}]")));
                        }
                        if spans.is_empty() {
                            spans.push(text::Span::new(" "));
                        }
                        let paragraph = TextParagraph::with_spans(text::Text {
                            content: spans.as_slice(),
                            bounds: Size::new(600.0, f32::INFINITY),
                            size: 13.0.into(),
                            line_height: text::LineHeight::Relative(1.4),
                            font: iced::Font::MONOSPACE,
                            horizontal_alignment: iced::alignment::Horizontal::Left,
                            vertical_alignment: iced::alignment::Vertical::Top,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::WordOrGlyph,
                        });
                        paragraph.min_height().max(18.2) + 4.0
                    })
                    .fold(0.0, f32::max)
            })
            .collect();
        let rows = bounds(&heights);
        let total_height: f32 = heights.iter().sum();
        let mut maximum = 0;
        for step in 0..1000 {
            let top = (total_height - 800.0).max(0.0) * step as f32 / 999.0;
            let range = visible_range(&rows, top, top + 800.0);
            let expected: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, row)| row.y + row.height >= top && row.y <= top + 800.0)
                .map(|(index, _)| index)
                .collect();
            maximum = maximum.max(range.len());
            assert_eq!(range.collect::<Vec<_>>(), expected);
        }
        eprintln!(
            "{} sample rows; at most {} rows visited per 800px viewport",
            rows.len(),
            maximum
        );
        assert!(maximum < 100);
    }
}
