//! A pinned, document-wide preview using the actual laid-out diff row heights.
use iced::advanced::{mouse, renderer, text, Renderer as _, Shell};
use iced::{alignment, event, Border, Event, Font, Point, Rectangle, Renderer};

use crate::app::Message;
use crate::diff::{CellKind, DiffCell, DiffData};
use crate::theme::{blend, DiffTheme};

pub const WIDTH: f32 = 132.0;
const SCROLLBAR_SPACE: f32 = 14.0;
const VIEWPORT_HEIGHT: f32 = 128.0;

pub struct Minimap<'a> {
    diff: &'a DiffData,
    theme: &'a DiffTheme,
    font: Font,
    font_size: f32,
    tab_width: usize,
}

#[derive(Default)]
pub struct State {
    bands: Vec<Band>,
    grabbed_at: Option<f32>,
}

struct Band {
    side: usize,
    top: f32,
    bottom: f32,
    kind: CellKind,
}

#[derive(Debug, Clone, Copy)]
struct Geometry {
    bounds: Rectangle,
    scale: f32,
    document_height: f32,
    viewport_height: f32,
    offset: f32,
}

impl Geometry {
    fn new(content: Rectangle, viewport: &Rectangle) -> Self {
        let width = WIDTH.min(content.width);
        Self {
            bounds: Rectangle {
                x: content.x + content.width - width,
                y: viewport.y,
                width: (width - SCROLLBAR_SPACE).max(0.0),
                height: viewport.height,
            },
            scale: if content.height > 0.0 {
                (viewport.height / content.height).min(0.15)
            } else {
                0.0
            },
            document_height: content.height,
            viewport_height: viewport.height,
            offset: (viewport.y - content.y).max(0.0),
        }
    }

    fn scroll_range(self) -> f32 {
        (self.document_height - self.viewport_height).max(0.0)
    }

    fn thumb(self) -> Rectangle {
        let document_height = self.document_height * self.scale;
        // Keep the drag target easy to grab at every document size and zoom.
        // Short previews cap it at their full height so it stays inside the map.
        let height = VIEWPORT_HEIGHT.min(document_height);
        let fraction = if self.scroll_range() > 0.0 {
            (self.offset / self.scroll_range()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Rectangle {
            y: self.bounds.y + fraction * (document_height - height),
            height,
            ..self.bounds
        }
    }

    fn seek(self, y: f32, grabbed_at: f32) -> f32 {
        let travel = self.document_height * self.scale - self.thumb().height;
        if self.scroll_range() <= 0.0 || travel <= 0.0 {
            return 0.0;
        }
        ((y - self.bounds.y - grabbed_at) / travel).clamp(0.0, 1.0)
    }
}

impl<'a> Minimap<'a> {
    pub fn new(
        diff: &'a DiffData,
        theme: &'a DiffTheme,
        font: Font,
        font_size: f32,
        tab_width: usize,
    ) -> Self {
        Self {
            diff,
            theme,
            font,
            font_size,
            tab_width,
        }
    }

    pub fn layout(&self, state: &mut State, rows: &[Rectangle]) {
        // Merge adjacent changes once per layout so scrolling does not traverse
        // every row, even when a large file is entirely inserted or deleted.
        state.bands.clear();
        for side in 0..2 {
            for (row, bounds) in self.diff.rows.iter().zip(rows) {
                let cell = if side == 0 { &row.left } else { &row.right };
                if !matches!(cell.kind, CellKind::Delete | CellKind::Insert) {
                    continue;
                }
                if let Some(previous) = state.bands.last_mut() {
                    if previous.side == side
                        && previous.kind == cell.kind
                        && previous.bottom == bounds.y
                    {
                        previous.bottom = bounds.y + bounds.height;
                        continue;
                    }
                }
                state.bands.push(Band {
                    side,
                    top: bounds.y,
                    bottom: bounds.y + bounds.height,
                    kind: cell.kind.clone(),
                });
            }
        }
    }

    pub fn on_event(
        &self,
        state: &mut State,
        event: &Event,
        content: Rectangle,
        viewport: &Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let geometry = Geometry::new(content, viewport);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(geometry.bounds) =>
            {
                let y = cursor.position().unwrap().y;
                let thumb = geometry.thumb();
                let grabbed_at = if y >= thumb.y && y <= thumb.y + thumb.height {
                    y - thumb.y
                } else {
                    thumb.height / 2.0
                };
                state.grabbed_at = Some(grabbed_at);
                shell.publish(Message::MinimapScroll(geometry.seek(y, grabbed_at)));
                event::Status::Captured
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.grabbed_at.is_some() => {
                shell.publish(Message::MinimapScroll(
                    geometry.seek(position.y + geometry.offset, state.grabbed_at.unwrap()),
                ));
                event::Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.grabbed_at.take().is_some() =>
            {
                event::Status::Captured
            }
            Event::Mouse(mouse::Event::CursorLeft)
            | Event::Window(iced::window::Event::Unfocused) => {
                state.grabbed_at = None;
                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    pub fn is_over(&self, content: Rectangle, viewport: &Rectangle, cursor: mouse::Cursor) -> bool {
        cursor.is_over(Geometry::new(content, viewport).bounds)
    }

    pub fn draw(
        &self,
        state: &State,
        rows: &[Rectangle],
        content: Rectangle,
        viewport: &Rectangle,
        renderer: &mut Renderer,
    ) {
        let geometry = Geometry::new(content, viewport);
        let bounds = geometry.bounds;
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }
        renderer.with_layer(bounds, |renderer| {
            fill(
                renderer,
                bounds,
                blend(self.theme.background, self.theme.subheader_bg, 0.5),
            );
            fill(
                renderer,
                Rectangle {
                    width: 1.0,
                    ..bounds
                },
                self.theme.divider,
            );
            let inner = Rectangle {
                x: bounds.x + 5.0,
                width: (bounds.width - 10.0).max(0.0),
                ..bounds
            };
            if inner.width <= 4.0 {
                return;
            }
            let pane_width = inner.width / 2.0;
            fill(
                renderer,
                Rectangle {
                    x: inner.x + pane_width,
                    width: 1.0,
                    ..inner
                },
                self.theme.divider,
            );
            let height = (content.height * geometry.scale).min(bounds.height);
            let band_step = 1.0_f32.max(height / 2048.0);
            let split = state.bands.partition_point(|band| band.side == 0);
            for (side, bands) in [&state.bands[..split], &state.bands[split..]]
                .into_iter()
                .enumerate()
            {
                let mut y = 0.0;
                while y < height && geometry.scale > 0.0 {
                    let top = y / geometry.scale;
                    let bottom = (y + band_step) / geometry.scale;
                    let index = bands.partition_point(|band| band.bottom <= top);
                    if let Some(band) = bands.get(index).filter(|band| band.top < bottom) {
                        fill(
                            renderer,
                            Rectangle {
                                x: inner.x + side as f32 * pane_width,
                                y: bounds.y + y,
                                width: pane_width,
                                height: band_step,
                            },
                            match band.kind {
                                CellKind::Delete => self.theme.delete_bg,
                                _ => self.theme.insert_bg,
                            },
                        );
                    }
                    y += band_step;
                }
            }
            // At most one sampled row per 1.5px. Binary search preserves the
            // mapping for wrapped rows without drawing thousands of offscreen lines.
            let mut previous = None;
            let step = 1.5_f32.max(height / 2048.0);
            let mut y = 0.0;
            while y < height && geometry.scale > 0.0 {
                let source_y = y / geometry.scale;
                let index = rows.partition_point(|row| row.y + row.height <= source_y);
                if previous != Some(index) {
                    if let Some(row) = self.diff.rows.get(index) {
                        for (side, cell) in [&row.left, &row.right].into_iter().enumerate() {
                            let clip = Rectangle {
                                x: inner.x + side as f32 * pane_width,
                                width: pane_width - 2.0,
                                ..inner
                            };
                            self.draw_cell(
                                cell,
                                clip,
                                bounds.y + rows[index].y * geometry.scale,
                                (self.font_size * geometry.scale).clamp(1.1, 2.4),
                                renderer,
                            );
                        }
                    }
                    previous = Some(index);
                }
                y += step;
            }
            let thumb = geometry.thumb();
            if thumb.height > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: thumb,
                        border: Border {
                            color: blend(self.theme.background, self.theme.text, 0.35),
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    },
                    self.theme.text.scale_alpha(0.10),
                );
            }
        });
    }

    fn draw_cell(
        &self,
        cell: &DiffCell,
        clip: Rectangle,
        y: f32,
        size: f32,
        renderer: &mut Renderer,
    ) {
        use text::Renderer as _;
        // Bound work for huge individual lines. Preserve indentation and tab stops.
        let mut label = String::new();
        let mut column = 0;
        for ch in cell
            .spans
            .iter()
            .flat_map(|span| span.text.chars())
            .take(160)
        {
            if ch == '\t' {
                let spaces = self.tab_width - column % self.tab_width;
                label.extend(std::iter::repeat_n(' ', spaces));
                column += spaces;
            } else if ch != '\r' && ch != '\n' {
                label.push(ch);
                column += 1;
            }
        }
        if label.trim().is_empty() {
            return;
        }
        renderer.fill_text(
            text::Text {
                content: label,
                bounds: clip.size(),
                size: size.into(),
                line_height: text::LineHeight::Relative(1.0),
                font: self.font,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
            },
            Point::new(clip.x, y),
            blend(self.theme.background, self.theme.text, 0.65),
            clip,
        );
    }
}

fn fill(renderer: &mut Renderer, bounds: Rectangle, color: iced::Color) {
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            ..Default::default()
        },
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geometry(height: f32, viewport_height: f32, offset: f32) -> Geometry {
        Geometry::new(
            Rectangle {
                x: 10.0,
                y: 50.0,
                width: 1000.0,
                height,
            },
            &Rectangle {
                x: 10.0,
                y: 50.0 + offset,
                width: 1000.0,
                height: viewport_height,
            },
        )
    }

    #[test]
    fn viewport_and_seek_match_at_top_middle_and_bottom() {
        for offset in [0.0, 4500.0, 9000.0] {
            let geometry = geometry(10_000.0, 1000.0, offset);
            let thumb = geometry.thumb();
            assert_eq!(thumb.height, 128.0);
            let fraction = geometry.seek(thumb.y + thumb.height / 2.0, thumb.height / 2.0);
            assert!((fraction - offset / 9000.0).abs() < 0.0001);
            assert_eq!(geometry.seek(geometry.bounds.y - 1000.0, 0.0), 0.0);
            assert_eq!(geometry.seek(geometry.bounds.y + 2000.0, 0.0), 1.0);
        }
    }

    #[test]
    fn fixed_thumb_stays_in_bounds_for_very_large_files() {
        let geometry = geometry(10_000_000.0, 800.0, 9_999_200.0);
        let thumb = geometry.thumb();
        assert_eq!(thumb.height, 128.0);
        assert!((thumb.y + thumb.height - geometry.bounds.y - geometry.bounds.height).abs() < 0.01);
        assert_eq!(geometry.seek(thumb.y + 4.0, 4.0), 1.0);
    }

    #[test]
    fn highlight_height_is_fixed_across_document_and_viewport_sizes() {
        for height in [10_000.0, 100_000.0, 10_000_000.0] {
            for viewport_height in [400.0, 800.0, 1200.0] {
                let geometry = geometry(height, viewport_height, 0.0);
                assert_eq!(geometry.thumb().height, 128.0);
            }
        }
    }

    #[test]
    fn highlight_fits_short_previews_and_small_windows() {
        for (height, viewport_height) in [(25.0, 800.0), (10_000.0, 80.0)] {
            let geometry = geometry(height, viewport_height, 0.0);
            let thumb = geometry.thumb();
            assert!(thumb.height <= 128.0);
            assert!(thumb.height <= geometry.bounds.height);
            assert!(thumb.y + thumb.height <= geometry.bounds.y + geometry.bounds.height);
        }
    }

    #[test]
    fn empty_and_short_files_have_no_scroll_range() {
        for height in [0.0, 25.0, 500.0] {
            let geometry = geometry(height, 800.0, 0.0);
            assert_eq!(geometry.scroll_range(), 0.0);
            assert_eq!(geometry.seek(500.0, 0.0), 0.0);
            assert!(geometry.thumb().height.is_finite());
        }
    }

    #[test]
    fn change_bands_use_wrapped_row_heights_and_merge_adjacent_changes() {
        let diff = crate::diff::compute_diff("a\nb\nc\nd\n", "a\nchanged b\nchanged c\nd\n");
        let theme = crate::theme::default_diff_theme(&iced::Theme::Dark);
        let minimap = Minimap::new(&diff, &theme, Font::MONOSPACE, 13.0, 4);
        let heights = [20.0, 100.0, 40.0, 20.0];
        assert_eq!(diff.rows.len(), heights.len());
        let mut y = 0.0;
        let rows: Vec<_> = heights
            .into_iter()
            .map(|height| {
                let bounds = Rectangle {
                    x: 0.0,
                    y,
                    width: 1000.0,
                    height,
                };
                y += height;
                bounds
            })
            .collect();
        let mut state = State::default();
        minimap.layout(&mut state, &rows);
        assert_eq!(state.bands.len(), 2);
        for (side, band) in state.bands.iter().enumerate() {
            assert_eq!(band.side, side);
            assert_eq!((band.top, band.bottom), (20.0, 160.0));
        }
        minimap.layout(&mut state, &rows);
        assert_eq!(state.bands.len(), 2, "layout must replace stale bands");
    }

    #[test]
    fn dragging_preserves_grab_position_after_scroll_translation() {
        let diff = DiffData::default();
        let theme = crate::theme::default_diff_theme(&iced::Theme::Dark);
        let minimap = Minimap::new(&diff, &theme, Font::MONOSPACE, 13.0, 4);
        let geometry = geometry(10_000.0, 1000.0, 4500.0);
        let mut state = State {
            grabbed_at: Some(20.0),
            ..Default::default()
        };
        let content = Rectangle {
            x: 10.0,
            y: 50.0,
            width: 1000.0,
            height: 10_000.0,
        };
        let viewport = Rectangle {
            y: 4550.0,
            height: 1000.0,
            ..content
        };
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        // Mouse event coordinates are window-relative; Scrollable translates only
        // the Cursor. The minimap must compensate for that when dragging.
        let position = Point::new(950.0, geometry.thumb().y - geometry.offset + 20.0);
        let status = minimap.on_event(
            &mut state,
            &Event::Mouse(mouse::Event::CursorMoved { position }),
            content,
            &viewport,
            mouse::Cursor::Available(Point::new(position.x, position.y + geometry.offset)),
            &mut shell,
        );
        assert_eq!(status, event::Status::Captured);
        assert_eq!(messages, vec![Message::MinimapScroll(0.5)]);
    }
}
