//! Scroll to laid-out rows instead of estimating heights (which vary with wrapping).
use iced::advanced::widget::{self, operation, Id, Operation};
use iced::{Rectangle, Task, Vector};

use crate::app::Message;

pub const DIFF_SCROLL_ID: &str = "diff-scroll";

pub fn row_id(row: usize) -> String {
    format!("diff-row-{row}")
}

pub fn scroll_to_row(row: usize) -> Task<Message> {
    widget::operate(FindRow {
        target: Id::new(row_id(row)),
        scroll: Id::new(DIFF_SCROLL_ID),
        content_bounds: None,
        row_bounds: None,
    })
}

struct FindRow {
    target: Id,
    scroll: Id,
    content_bounds: Option<Rectangle>,
    row_bounds: Option<Rectangle>,
}

impl Operation<Message> for FindRow {
    fn container(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
    ) {
        if id == Some(&self.target) {
            self.row_bounds = Some(bounds);
        }
        operate_on_children(self);
    }

    fn scrollable(
        &mut self,
        _state: &mut dyn operation::Scrollable,
        id: Option<&Id>,
        _bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
    ) {
        if id == Some(&self.scroll) {
            self.content_bounds = Some(content_bounds);
        }
    }

    fn finish(&self) -> operation::Outcome<Message> {
        match (self.content_bounds, self.row_bounds) {
            (Some(content), Some(row)) => {
                operation::Outcome::Chain(Box::new(operation::scrollable::scroll_to(
                    self.scroll.clone(),
                    operation::scrollable::AbsoluteOffset {
                        x: 0.0,
                        y: (row.y - content.y).max(0.0),
                    },
                )))
            }
            _ => operation::Outcome::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use operation::scrollable::{AbsoluteOffset, RelativeOffset};

    #[derive(Default)]
    struct ScrollState {
        offset: AbsoluteOffset,
    }

    impl operation::Scrollable for ScrollState {
        fn snap_to(&mut self, _offset: RelativeOffset) {
            unreachable!();
        }

        fn scroll_to(&mut self, offset: AbsoluteOffset) {
            self.offset = offset;
        }

        fn scroll_by(&mut self, _offset: AbsoluteOffset, _bounds: Rectangle, _content: Rectangle) {
            unreachable!();
        }
    }

    #[test]
    fn navigation_uses_layout_bounds_and_ignores_nested_horizontal_scrollers() {
        let mut operation = FindRow {
            target: Id::new(row_id(3)),
            scroll: Id::new(DIFF_SCROLL_ID),
            content_bounds: None,
            row_bounds: None,
        };
        let mut state = ScrollState::default();
        let content = Rectangle {
            y: 100.0,
            height: 2000.0,
            ..Rectangle::default()
        };
        operation.scrollable(
            &mut state,
            Some(&Id::new(DIFF_SCROLL_ID)),
            Rectangle::default(),
            content,
            Vector::new(0.0, 200.0),
        );
        // A wrapped row can be much farther down than its line number suggests.
        operation.container(
            Some(&Id::new(row_id(3))),
            Rectangle {
                y: 780.0,
                ..Rectangle::default()
            },
            &mut |_| {},
        );
        operation.scrollable(
            &mut state,
            None,
            Rectangle::default(),
            Rectangle::default(),
            Vector::ZERO,
        );
        let operation::Outcome::Chain(mut scroll) = operation.finish() else {
            panic!("Expected a scroll operation");
        };
        scroll.scrollable(
            &mut state,
            Some(&Id::new(DIFF_SCROLL_ID)),
            Rectangle::default(),
            content,
            Vector::ZERO,
        );
        assert_eq!(state.offset, AbsoluteOffset { x: 0.0, y: 680.0 });
    }

    #[test]
    fn missing_row_does_not_scroll() {
        let operation = FindRow {
            target: Id::new(row_id(3)),
            scroll: Id::new(DIFF_SCROLL_ID),
            content_bounds: Some(Rectangle::default()),
            row_bounds: None,
        };
        assert!(matches!(operation.finish(), operation::Outcome::None));
    }
}
