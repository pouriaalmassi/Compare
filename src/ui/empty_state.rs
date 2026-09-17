use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, LineDash, Path, Stroke};
use iced::widget::{button, center, column, container, stack, svg, text};
use iced::{font, Alignment, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;
use crate::theme::{blend, DiffTheme};

fn make_docs_icon_svg(is_dark: bool) -> String {
    let (stroke, fill) = if is_dark {
        ("#a0a5b5", "#282a36")
    } else {
        ("#767676", "#ffffff")
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 14 86 106" fill="none" stroke="{stroke}" stroke-width="5.5" stroke-linecap="round" stroke-linejoin="round">
  <path d="M36 40V28C36 23.5 39.5 20 44 20H63L90 47V85C90 89.5 86.5 93 82 93H70" />
  <path d="M63 20V41C63 44.5 65.5 47 69 47H90" />
  <path d="M16 30C16 25.5 19.5 22 24 22H43L70 49V105C70 109.5 66.5 113 62 113H24C19.5 113 16 109.5 16 105V30Z" fill="{fill}" />
  <path d="M43 22V43C43 46.5 45.5 49 49 49H70" />
</svg>"##,
        stroke = stroke,
        fill = fill
    )
}

#[derive(Debug, Clone, Copy)]
pub struct DashedBorder {
    pub is_hovering: bool,
    pub stroke_color: Color,
    pub hover_color: Color,
}

impl canvas::Program<Message, Theme, Renderer> for DashedBorder {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let inset = 20.0;
        let radius = 16.0;
        let rect_w = (bounds.width - 2.0 * inset).max(0.0);
        let rect_h = (bounds.height - 2.0 * inset).max(0.0);

        if rect_w > 0.0 && rect_h > 0.0 {
            let path = Path::rounded_rectangle(
                Point::new(inset, inset),
                Size::new(rect_w, rect_h),
                radius.into(),
            );

            let stroke_color = if self.is_hovering {
                self.hover_color
            } else {
                self.stroke_color
            };

            frame.stroke(
                &path,
                Stroke {
                    style: stroke_color.into(),
                    width: if self.is_hovering { 2.0 } else { 1.5 },
                    line_dash: LineDash {
                        segments: &[6.0, 6.0],
                        offset: 0,
                    },
                    ..Stroke::default()
                },
            );
        }

        vec![frame.into_geometry()]
    }
}

pub fn view_empty_state<'a>(
    file_1_name: Option<&str>,
    is_hovering: bool,
    status_message: Option<&str>,
    diff_theme: &'a DiffTheme,
    is_dark: bool,
) -> Element<'a, Message> {
    let border_color = if is_dark {
        blend(diff_theme.text, diff_theme.background, 0.3)
    } else {
        Color::from_rgb8(185, 185, 185)
    };
    let hover_color = Color::from_rgb8(37, 99, 235);

    let canvas_border = Canvas::new(DashedBorder {
        is_hovering,
        stroke_color: border_color,
        hover_color,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    let svg_content = make_docs_icon_svg(is_dark);
    let svg_handle = svg::Handle::from_memory(svg_content.into_bytes());
    let icon = svg(svg_handle).width(44).height(54);

    let (title, subtitle) = if let Some(name) = file_1_name {
        (
            "Drag Second File Here",
            format!("Loaded \"{}\" — drop second file to compare", name),
        )
    } else {
        (
            "Drag Two Text Files Here",
            "Two plain text files, dropped together or one at a time".to_string(),
        )
    };

    let heading = text(title)
        .size(18)
        .font(font::Font {
            weight: font::Weight::Bold,
            ..Default::default()
        })
        .color(diff_theme.text);

    let sub = text(subtitle)
        .size(14)
        .color(blend(diff_theme.text, diff_theme.background, 0.6));

    let text_color = diff_theme.text;
    let btn_bg = diff_theme.button_bg;
    let btn_border = diff_theme.button_border;

    let choose_button = button(
        text("Choose Files...")
            .size(13)
            .font(font::Font {
                weight: font::Weight::Medium,
                ..Default::default()
            })
            .color(text_color),
    )
    .padding([7, 18])
    .on_press(Message::ChooseFiles)
    .style(move |_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered => blend(text_color, btn_bg, 0.08),
            button::Status::Pressed => blend(text_color, btn_bg, 0.15),
            _ => btn_bg,
        };
        button::Style {
            background: Some(bg.into()),
            text_color,
            border: Border {
                color: btn_border,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    });

    let mut center_col = column![icon, heading, sub]
        .spacing(12)
        .align_x(Alignment::Center);

    center_col = center_col.push(choose_button);

    if let Some(err) = status_message {
        center_col = center_col.push(
            text(err.to_string())
                .size(13)
                .color(diff_theme.delete_line_num),
        );
    }

    let center_content = container(center(center_col))
        .width(Length::Fill)
        .height(Length::Fill);

    let bg_color = if is_hovering {
        blend(Color::from_rgb8(37, 99, 235), diff_theme.empty_bg, 0.05)
    } else {
        diff_theme.empty_bg
    };

    container(
        stack![
            canvas_border,
            center_content,
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        ..Default::default()
    })
    .into()
}
