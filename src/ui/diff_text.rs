//! Diff text with whitespace markers painted over the shaped whitespace bounds.
use iced::advanced::text::{Paragraph, Renderer as TextRenderer};
use iced::advanced::{layout, mouse, renderer, text, widget::tree, Layout, Widget};
use iced::{
    alignment, Border, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme,
};

use crate::app::Message;
use crate::diff::{CellKind, DiffCell};
use crate::theme::{blend, DiffTheme};
use crate::whitespace::{display_spans, Marker, WhitespaceMode};

type TextParagraph = <Renderer as TextRenderer>::Paragraph;

#[derive(Default)]
struct State {
    paragraph: TextParagraph,
    spans: Vec<text::Span<'static, (), Font>>,
}

pub struct DiffText {
    spans: Vec<text::Span<'static, (), Font>>,
    markers: Vec<Option<Marker>>,
    font: Font,
    size: f32,
    wrap: bool,
    show_markers: bool,
    color: Color,
    marker_color: Color,
}

impl DiffText {
    pub fn new(
        cell: &DiffCell,
        theme: &DiffTheme,
        font: Font,
        size: f32,
        wrap: bool,
        mode: WhitespaceMode,
        tab_width: usize,
    ) -> Self {
        let highlight = match cell.kind {
            CellKind::Delete => theme.delete_highlight,
            CellKind::Insert => theme.insert_highlight,
            _ => theme.background,
        };
        let mut spans = Vec::new();
        let mut markers = Vec::new();
        for piece in display_spans(cell, tab_width) {
            let mut span = text::Span::new(piece.text);
            if piece.emphasized {
                span = span.background(highlight);
            }
            spans.push(span);
            markers.push(piece.marker);
        }
        if let Some(marker) = cell.newline_marker {
            spans.push(text::Span::new(format!("  [{marker}]")).color(theme.equal_line_num));
            markers.push(None);
        }
        // Keep truly empty lines at the same line height without a fake space marker.
        if spans.is_empty() {
            spans.push(text::Span::new(" "));
            markers.push(None);
        }
        Self {
            spans,
            markers,
            font,
            size,
            wrap,
            show_markers: mode.shows(&cell.kind),
            color: theme.text,
            marker_color: blend(theme.text, theme.background, 0.4),
        }
    }
}

impl Widget<Message, Theme, Renderer> for DiffText {
    fn size(&self) -> Size<Length> {
        Size::new(
            if self.wrap {
                Length::Fill
            } else {
                Length::Shrink
            },
            Length::Shrink,
        )
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        layout::sized(limits, self.size().width, Length::Shrink, |limits| {
            let settings = text::Text {
                content: (),
                bounds: limits.max(),
                size: self.size.into(),
                line_height: text::LineHeight::Relative(1.4),
                font: self.font,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: if self.wrap {
                    text::Wrapping::WordOrGlyph
                } else {
                    text::Wrapping::None
                },
            };
            let difference = state.paragraph.compare(settings);
            if state.spans != self.spans || matches!(difference, text::Difference::Shape) {
                state.paragraph = TextParagraph::with_spans(text::Text {
                    content: self.spans.as_slice(),
                    bounds: settings.bounds,
                    size: settings.size,
                    line_height: settings.line_height,
                    font: settings.font,
                    horizontal_alignment: settings.horizontal_alignment,
                    vertical_alignment: settings.vertical_alignment,
                    shaping: settings.shaping,
                    wrapping: settings.wrapping,
                });
                state.spans = self.spans.clone();
            } else if matches!(difference, text::Difference::Bounds) {
                state.paragraph.resize(settings.bounds);
            }
            state.paragraph.min_bounds()
        })
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        let translation = layout.position() - Point::ORIGIN;
        use iced::advanced::Renderer as _;
        renderer.with_layer(clip, |renderer| {
            for (index, span) in self.spans.iter().enumerate() {
                if let Some(highlight) = span.highlight {
                    for bounds in state.paragraph.span_bounds(index) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: bounds + translation,
                                ..Default::default()
                            },
                            highlight.background,
                        );
                    }
                }
            }
            renderer.fill_paragraph(&state.paragraph, layout.position(), self.color, clip);
            if self.show_markers {
                for (index, marker) in self.markers.iter().enumerate() {
                    if let Some(marker) = marker {
                        for bounds in state.paragraph.span_bounds(index) {
                            draw_marker(
                                renderer,
                                *marker,
                                bounds + translation,
                                self.marker_color,
                                self.size,
                            );
                        }
                    }
                }
            }
        });
    }
}

fn draw_marker(
    renderer: &mut Renderer,
    marker: Marker,
    bounds: Rectangle,
    color: Color,
    size: f32,
) {
    use iced::advanced::Renderer as _;
    let stroke = (size / 13.0).max(1.0);
    let y = bounds.center_y();
    if marker == Marker::Space {
        let diameter = stroke * 1.8;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    Point::new(bounds.center_x() - diameter / 2.0, y - diameter / 2.0),
                    Size::new(diameter, diameter),
                ),
                border: Border {
                    radius: diameter.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            color,
        );
    } else if bounds.width > stroke * 2.0 {
        let left = bounds.x + stroke;
        let right = bounds.x + bounds.width - stroke * 2.0;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    Point::new(left, y - stroke / 2.0),
                    Size::new((right - left).max(stroke), stroke),
                ),
                ..Default::default()
            },
            color,
        );
        for step in 0..3 {
            let offset = step as f32 * stroke;
            for sign in [-1.0, 1.0] {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(right - offset, y + sign * offset - stroke / 2.0),
                            Size::new(stroke, stroke),
                        ),
                        ..Default::default()
                    },
                    color,
                );
            }
        }
    }
}

impl<'a> From<DiffText> for Element<'a, Message> {
    fn from(value: DiffText) -> Self {
        Element::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::compute_diff;

    fn paragraph(widget: &DiffText, width: f32) -> TextParagraph {
        TextParagraph::with_spans(text::Text {
            content: &widget.spans,
            bounds: Size::new(width, f32::INFINITY),
            size: widget.size.into(),
            line_height: text::LineHeight::Relative(1.4),
            font: widget.font,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: if widget.wrap {
                text::Wrapping::WordOrGlyph
            } else {
                text::Wrapping::None
            },
        })
    }

    #[test]
    fn markers_do_not_change_text_layout_or_insert_copyable_symbols() {
        let theme = crate::theme::default_diff_theme(&Theme::Dark);
        let diff = compute_diff("", "alpha beta\tlongword0123456789  \n");
        let cell = &diff.rows[0].right;
        for wrap in [false, true] {
            for size in [8.0, 13.0, 32.0] {
                let off = DiffText::new(
                    cell,
                    &theme,
                    Font::MONOSPACE,
                    size,
                    wrap,
                    WhitespaceMode::Off,
                    4,
                );
                let all = DiffText::new(
                    cell,
                    &theme,
                    Font::MONOSPACE,
                    size,
                    wrap,
                    WhitespaceMode::All,
                    4,
                );
                assert_eq!(off.spans, all.spans);
                assert_eq!(
                    paragraph(&off, 100.0).min_bounds(),
                    paragraph(&all, 100.0).min_bounds()
                );
                assert!(all.spans.iter().all(|span| !span.text.contains(['·', '→'])));
            }
        }
    }

    #[test]
    fn shaped_bounds_include_leading_trailing_spaces_and_tabs() {
        let theme = crate::theme::default_diff_theme(&Theme::Dark);
        let diff = compute_diff("", " \tx  \n");
        let widget = DiffText::new(
            &diff.rows[0].right,
            &theme,
            Font::MONOSPACE,
            13.0,
            false,
            WhitespaceMode::All,
            4,
        );
        let shaped = paragraph(&widget, 500.0);
        let mut space_width = None;
        for (index, marker) in widget.markers.iter().enumerate() {
            if let Some(marker) = marker {
                let bounds = shaped.span_bounds(index);
                assert_eq!(bounds.len(), 1);
                assert!(bounds[0].width > 0.0);
                assert!(bounds[0].x + bounds[0].width <= shaped.min_width() + 0.1);
                match marker {
                    Marker::Space => {
                        space_width = Some(bounds[0].width);
                    }
                    Marker::Tab => {
                        assert!((bounds[0].width - space_width.unwrap() * 3.0).abs() < 0.1);
                    }
                }
            }
        }
    }
}
