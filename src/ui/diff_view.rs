use iced::widget::{
    button, column, container, horizontal_space, rich_text, row, scrollable, span, stack, text,
    toggler,
};
use iced::{alignment, font, Alignment, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::diff::{CellKind, DiffCell, DiffData};
use crate::theme::{blend, DiffTheme};

#[allow(clippy::too_many_arguments)]
pub fn view_diff<'a>(
    diff: &'a DiffData,
    file_1_name: Option<&'a str>,
    file_2_name: Option<&'a str>,
    wrap_lines: bool,
    is_hovering: bool,
    status_message: Option<&'a str>,
    diff_theme: &'a DiffTheme,
    font_size: f32,
) -> Element<'a, Message> {
    let top_bar = render_top_bar(diff, status_message, wrap_lines, diff_theme);
    let left_name = file_1_name.unwrap_or("File 1");
    let right_name = file_2_name.unwrap_or("File 2");

    let subheader_divider = container(horizontal_space())
        .width(1)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(diff_theme.divider.into()),
            ..Default::default()
        });

    let subheader = row![
        container(
            text(left_name)
                .size(13)
                .font(font::Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                })
                .color(diff_theme.text),
        )
        .width(Length::FillPortion(1))
        .padding([4, 12]),
        subheader_divider,
        container(
            text(right_name)
                .size(13)
                .font(font::Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                })
                .color(diff_theme.text),
        )
        .width(Length::FillPortion(1))
        .padding([4, 12]),
    ]
    .height(30)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let subheader_container = container(subheader)
        .width(Length::Fill)
        .height(30)
        .style(move |_| container::Style {
            background: Some(diff_theme.subheader_bg.into()),
            ..Default::default()
        });

    let divider = container(horizontal_space())
        .width(Length::Fill)
        .height(1)
        .style(move |_| container::Style {
            background: Some(diff_theme.divider.into()),
            ..Default::default()
        });

    let mut diff_rows_column = column![].spacing(0).width(Length::Fill);

    for row_data in &diff.rows {
        let left_cell = render_cell(&row_data.left, wrap_lines, diff_theme, font_size);
        let right_cell = render_cell(&row_data.right, wrap_lines, diff_theme, font_size);

        let row_item = row![left_cell, right_cell]
            .spacing(1)
            .width(Length::Fill);

        diff_rows_column = diff_rows_column.push(row_item);
    }

    let diff_scroll = scrollable(diff_rows_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let background_center_line = row![
        horizontal_space().width(Length::FillPortion(1)),
        container(horizontal_space())
            .width(1)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(diff_theme.divider.into()),
                ..Default::default()
            }),
        horizontal_space().width(Length::FillPortion(1)),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let diff_area = stack![
        background_center_line,
        diff_scroll,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![top_bar, subheader_container, divider, diff_area]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(diff_theme.background.into()),
        border: Border {
            color: if is_hovering {
                Color::from_rgb8(37, 99, 235)
            } else {
                Color::TRANSPARENT
            },
            width: if is_hovering { 2.0 } else { 0.0 },
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn render_top_bar<'a>(
    diff: &'a DiffData,
    status_message: Option<&str>,
    wrap_lines: bool,
    diff_theme: &'a DiffTheme,
) -> Element<'a, Message> {
    let clear_btn = subtle_button("Clear", Message::ClearAll, diff_theme);

    let status_info = if diff.differ_count == 0 {
        "Files are identical".to_string()
    } else if diff.differ_count == 1 {
        "1 line differs".to_string()
    } else {
        format!("{} lines differ", diff.differ_count)
    };

    let final_status = status_message.map(|s| s.to_string()).unwrap_or(status_info);

    let status_label = text(final_status)
        .size(13)
        .color(diff_theme.equal_line_num);

    let wrap_toggler = toggler(wrap_lines)
        .label("Wrap Lines")
        .size(16)
        .text_size(13)
        .on_toggle(Message::ToggleWrapLines);

    let swap_btn = subtle_button("Swap Sides", Message::SwapSides, diff_theme);
    let choose_btn = subtle_button("Choose Files...", Message::ChooseFiles, diff_theme);

    let right_controls = row![wrap_toggler, swap_btn, choose_btn]
        .spacing(12)
        .align_y(Alignment::Center);

    let center_label = container(status_label)
        .width(Length::Fill)
        .height(42)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    let controls = row![
        clear_btn,
        horizontal_space(),
        right_controls
    ]
    .padding([6, 14])
    .height(42)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let bar_content = stack![center_label, controls]
        .width(Length::Fill)
        .height(42);

    let divider = container(horizontal_space())
        .width(Length::Fill)
        .height(1)
        .style(move |_| container::Style {
            background: Some(diff_theme.divider.into()),
            ..Default::default()
        });

    column![bar_content, divider]
        .height(Length::Shrink)
        .into()
}

fn render_cell<'a>(
    cell: &'a DiffCell,
    wrap_lines: bool,
    diff_theme: &'a DiffTheme,
    font_size: f32,
) -> Element<'a, Message> {
    let (bg_color, line_num_color) = match cell.kind {
        CellKind::Equal => (Color::TRANSPARENT, diff_theme.equal_line_num),
        CellKind::Delete => (diff_theme.delete_bg, diff_theme.delete_line_num),
        CellKind::Insert => (diff_theme.insert_bg, diff_theme.insert_line_num),
        CellKind::Empty => (diff_theme.empty_cell_bg, Color::TRANSPARENT),
    };

    let line_num_str = match cell.line_number {
        Some(num) => num.to_string(),
        None => String::new(),
    };

    let line_num_width = (font_size * 3.5 + 8.0).max(46.0);

    let line_num_widget = container(
        text(line_num_str)
            .size(font_size)
            .line_height(text::LineHeight::Relative(1.4))
            .font(iced::Font::MONOSPACE)
            .color(line_num_color)
            .wrapping(text::Wrapping::None),
    )
    .width(line_num_width)
    .align_x(alignment::Horizontal::Right)
    .padding(iced::Padding {
        top: 0.0,
        right: 10.0,
        bottom: 0.0,
        left: 4.0,
    });

    let text_content: Element<'a, Message> = if cell.spans.is_empty() {
        text(" ")
            .size(font_size)
            .line_height(text::LineHeight::Relative(1.4))
            .font(iced::Font::MONOSPACE)
            .into()
    } else {
        let span_elements: Vec<iced::widget::text::Span<'a, Message, iced::Font>> = cell
            .spans
            .iter()
            .map(|s| {
                let mut sp = span(s.text.as_str())
                    .size(font_size)
                    .font(iced::Font::MONOSPACE)
                    .color(diff_theme.text);

                if s.emphasized {
                    let highlight_bg = match cell.kind {
                        CellKind::Delete => diff_theme.delete_highlight,
                        CellKind::Insert => diff_theme.insert_highlight,
                        _ => blend(diff_theme.delete_highlight, diff_theme.insert_highlight, 0.5),
                    };
                    sp = sp.background(highlight_bg);
                }
                sp
            })
            .collect();

        if wrap_lines {
            rich_text(span_elements)
                .size(font_size)
                .line_height(text::LineHeight::Relative(1.4))
                .font(iced::Font::MONOSPACE)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .into()
        } else {
            scrollable(
                rich_text(span_elements)
                    .size(font_size)
                    .line_height(text::LineHeight::Relative(1.4))
                    .font(iced::Font::MONOSPACE)
                    .width(Length::Shrink)
                    .wrapping(text::Wrapping::None),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(0).scroller_width(0),
            ))
            .width(Length::Fill)
            .into()
        }
    };

    let content_row = row![line_num_widget, text_content]
        .spacing(4)
        .align_y(Alignment::Start)
        .width(Length::Fill);

    container(content_row)
        .width(Length::FillPortion(1))
        .padding([2, 4])
        .clip(!wrap_lines)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        })
        .into()
}

pub fn subtle_button<'a>(
    label: &'static str,
    message: Message,
    diff_theme: &DiffTheme,
) -> Element<'a, Message> {
    let text_color = diff_theme.text;
    let button_bg = diff_theme.button_bg;
    let button_border = diff_theme.button_border;

    button(text(label).size(13).color(text_color))
        .padding([5, 12])
        .on_press(message)
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => blend(text_color, button_bg, 0.08),
                button::Status::Pressed => blend(text_color, button_bg, 0.15),
                _ => button_bg,
            };
            button::Style {
                background: Some(bg.into()),
                text_color,
                border: Border {
                    color: button_border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}



