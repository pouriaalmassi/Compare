use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, scrollable, stack, svg, text,
    toggler, tooltip,
};
use iced::{alignment, font, Alignment, Border, Color, Element, Length, Theme};

use super::diff_text::DiffText;
use crate::app::Message;
use crate::diff::{CellKind, DiffCell, DiffData};
use crate::theme::{blend, DiffTheme};
use crate::whitespace::WhitespaceMode;

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
    diff_font: iced::Font,
    change_starts: &[usize],
    current_change: Option<usize>,
    whitespace: WhitespaceMode,
    tab_width: usize,
) -> Element<'a, Message> {
    let top_bar = render_top_bar(
        diff,
        status_message,
        wrap_lines,
        diff_theme,
        change_starts.len(),
        current_change,
        whitespace,
    );
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

    let subheader_container =
        container(subheader)
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

    for (index, row_data) in diff.rows.iter().enumerate() {
        let left_cell = render_cell(
            &row_data.left,
            wrap_lines,
            diff_theme,
            font_size,
            diff_font,
            whitespace,
            tab_width,
        );
        let right_cell = render_cell(
            &row_data.right,
            wrap_lines,
            diff_theme,
            font_size,
            diff_font,
            whitespace,
            tab_width,
        );

        let row_item = row![left_cell, right_cell].spacing(1).width(Length::Fill);

        let selected = current_change.and_then(|n| change_starts.get(n)) == Some(&index);
        diff_rows_column = diff_rows_column.push(
            container(row_item)
                .id(container::Id::new(super::navigation::row_id(index)))
                .style(move |_| container::Style {
                    border: Border {
                        color: diff_theme.text,
                        width: if selected { 1.0 } else { 0.0 },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );
    }

    let diff_scroll = scrollable(diff_rows_column)
        .id(scrollable::Id::new(super::navigation::DIFF_SCROLL_ID))
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

    let diff_area = stack![background_center_line, diff_scroll,]
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
    change_count: usize,
    current_change: Option<usize>,
    whitespace: WhitespaceMode,
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

    let status_label = tooltip(
        container(
            text(final_status.clone())
                .size(13)
                .color(diff_theme.equal_line_num)
                .wrapping(text::Wrapping::None),
        )
        .width(Length::Fill)
        .clip(true),
        text(final_status).size(13),
        tooltip::Position::Bottom,
    );

    let wrap_toggler = toggler(wrap_lines)
        .label("Wrap Lines")
        .size(16)
        .text_size(13)
        .on_toggle(Message::ToggleWrapLines);

    let swap_btn = icon_button(
        include_bytes!("../../assets/swap.svg"),
        "Swap sides (s)",
        Some(Message::SwapSides),
        diff_theme,
    );
    let choose_btn = subtle_button("Choose Files", Message::ChooseFiles, diff_theme);
    let previous = icon_button(
        include_bytes!("../../assets/prev.svg"),
        "Previous change (k)",
        (change_count > 0).then_some(Message::PreviousChange),
        diff_theme,
    );
    let next = icon_button(
        include_bytes!("../../assets/next.svg"),
        "Next change (j)",
        (change_count > 0).then_some(Message::NextChange),
        diff_theme,
    );
    let position = match current_change {
        Some(index) => format!("{} / {change_count}", index + 1),
        None => format!("{change_count} changes"),
    };
    let navigation = row![text(position).size(13), previous, next]
        .spacing(8)
        .align_y(Alignment::Center);
    let whitespace_picker = pick_list(
        WhitespaceMode::ALL,
        Some(whitespace),
        Message::SetWhitespace,
    )
    .text_size(13)
    .padding([5, 8]);
    let right_controls = row![
        horizontal_space(),
        whitespace_picker,
        wrap_toggler,
        navigation
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .width(Length::FillPortion(1));
    let left_controls = row![clear_btn, choose_btn, status_label]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(1));
    let bar_content = row![left_controls, swap_btn, right_controls]
        .spacing(16)
        .padding([6, 14])
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let divider = container(horizontal_space())
        .width(Length::Fill)
        .height(1)
        .style(move |_| container::Style {
            background: Some(diff_theme.divider.into()),
            ..Default::default()
        });

    column![bar_content, divider].height(Length::Shrink).into()
}

fn render_cell<'a>(
    cell: &'a DiffCell,
    wrap_lines: bool,
    diff_theme: &'a DiffTheme,
    font_size: f32,
    diff_font: iced::Font,
    whitespace: WhitespaceMode,
    tab_width: usize,
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
            .font(diff_font)
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

    let diff_text = DiffText::new(
        cell, diff_theme, diff_font, font_size, wrap_lines, whitespace, tab_width,
    );
    let text_content: Element<'a, Message> = if wrap_lines {
        diff_text.into()
    } else {
        scrollable(diff_text)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(0).scroller_width(0),
            ))
            .width(Length::Fill)
            .into()
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
    subtle_button_content(
        text(label).size(13).color(diff_theme.text).into(),
        Some(message),
        diff_theme,
    )
}

fn icon_button<'a>(
    bytes: &'static [u8],
    label: &'static str,
    message: Option<Message>,
    diff_theme: &DiffTheme,
) -> Element<'a, Message> {
    let icon_color = if message.is_some() {
        diff_theme.text
    } else {
        blend(diff_theme.text, diff_theme.button_bg, 0.4)
    };
    let icon = svg(svg::Handle::from_memory(bytes))
        .width(20)
        .height(20)
        .style(move |_, _| svg::Style {
            color: Some(icon_color),
        });
    tooltip(
        subtle_button_content(icon.into(), message, diff_theme),
        text(label).size(13),
        tooltip::Position::Bottom,
    )
    .into()
}

fn subtle_button_content<'a>(
    content: Element<'a, Message>,
    message: Option<Message>,
    diff_theme: &DiffTheme,
) -> Element<'a, Message> {
    let text_color = diff_theme.text;
    let button_bg = diff_theme.button_bg;
    let button_border = diff_theme.button_border;

    button(content)
        .padding([5, 12])
        .on_press_maybe(message)
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
