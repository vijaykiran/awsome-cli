use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, LoadingState};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_main_content(f, chunks[1], app);
    draw_footer(f, chunks[2], app);

    // Draw popups on top if active
    if app.show_service_popup {
        draw_service_popup(f, app);
    }

    if app.show_detail_popup {
        draw_detail_popup(f, app);
    }

    if app.show_quit_confirm {
        draw_quit_confirmation(f);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    // Create inner area (without borders)
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split into left and right sections
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length((app.profile_name.len() + 3) as u16), // Profile text + padding
        ])
        .split(inner_area);

    let favorites = app.get_favorite_services();

    // Left side - services
    let mut left_spans = vec![
        Span::styled("AWSOME ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| "),
    ];

    if favorites.is_empty() {
        left_spans.push(Span::styled("No favorites - Press Space to select service", Style::default().fg(Color::Gray)));
    } else {
        for (idx, (service_idx, service)) in favorites.iter().enumerate() {
            let is_active = *service_idx == app.active_service;

            let style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            left_spans.push(Span::styled(
                format!(" {} ", service.short_name()),
                style,
            ));

            if idx < favorites.len() - 1 {
                left_spans.push(Span::raw("• "));
            }
        }

        left_spans.push(Span::raw(" "));
        left_spans.push(Span::styled("[Space: More]", Style::default().fg(Color::DarkGray)));
    }

    // Right side - profile
    let profile_spans = vec![
        Span::styled("@ ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.profile_name, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ];

    // Render border
    let border = Block::default().borders(Borders::ALL);
    f.render_widget(border, area);

    // Render left content
    let left_paragraph = Paragraph::new(Line::from(left_spans));
    f.render_widget(left_paragraph, header_chunks[0]);

    // Render right content (profile)
    let right_paragraph = Paragraph::new(Line::from(profile_spans))
        .alignment(Alignment::Right);
    f.render_widget(right_paragraph, header_chunks[1]);
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &App) {
    // Determine color based on loading state
    let (title_color, border_style) = match app.loading_state {
        LoadingState::Loading => (Color::Yellow, Style::default().fg(Color::Yellow)),
        LoadingState::Error => (Color::Red, Style::default().fg(Color::Red)),
        LoadingState::Loaded => (Color::Green, Style::default().fg(Color::Green)),
        LoadingState::Idle => (Color::White, Style::default()),
    };

    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            // Check if it's a header/separator for S3 or IAM
            let is_header_or_sep = match app.get_active_service().service_type {
                crate::app::ServiceType::S3 => {
                    if i < app.s3_items.len() {
                        matches!(app.s3_items[i], crate::aws::S3Item::Header | crate::aws::S3Item::Separator)
                    } else {
                        false
                    }
                }
                crate::app::ServiceType::IAM => {
                    if i < app.iam_items.len() {
                        matches!(app.iam_items[i], crate::aws::IamItem::Header | crate::aws::IamItem::Separator)
                    } else {
                        false
                    }
                }
                crate::app::ServiceType::DynamoDB => {
                    if i < app.dynamodb_items.len() {
                        matches!(app.dynamodb_items[i], crate::aws::DynamoDbItem::Header | crate::aws::DynamoDbItem::Separator)
                    } else {
                        false
                    }
                }
                _ => false,
            };

            let style = if is_header_or_sep {
                 Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
                    .fg(if app.loading_state == LoadingState::Error {
                        Color::Red
                    } else {
                        Color::White
                    })
            } else {
                Style::default().fg(if app.loading_state == LoadingState::Error {
                    Color::LightRed
                } else {
                    Color::White
                })
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let loading_indicator = match app.loading_state {
        LoadingState::Loading => " [LOADING...]",
        LoadingState::Error => " [ERROR]",
        LoadingState::Loaded => " [READY]",
        LoadingState::Idle => "",
    };

    let title = format!("{}{}", app.get_active_service().as_str(), loading_indicator);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(title, Style::default().fg(title_color).add_modifier(Modifier::BOLD))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let status_color = match app.loading_state {
        LoadingState::Loading => Color::Yellow,
        LoadingState::Error => Color::Red,
        LoadingState::Loaded => Color::Green,
        LoadingState::Idle => Color::Cyan,
    };

    // Build status line with spinner on the left if loading
    let mut status_spans = Vec::new();

    if app.is_loading() {
        status_spans.push(Span::styled(
            format!("{} ", app.get_loading_spinner()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }

    status_spans.push(Span::styled(
        app.status_message.as_str(),
        Style::default().fg(status_color),
    ));

    let footer = Paragraph::new(Line::from(status_spans))
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(footer, area);
}

fn draw_service_popup(f: &mut Frame, app: &App) {
    // Calculate popup size and position (centered)
    let area = centered_rect(60, 60, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the popup container
    let popup_block = Block::default()
        .title("Select Service")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(popup_block, area);

    // Create inner area for content
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split inner area for list and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner_area);

    // Create service list items
    let items: Vec<ListItem> = app
        .services
        .iter()
        .enumerate()
        .map(|(i, service)| {
            let is_selected = i == app.popup_selected_index;
            let favorite_marker = if service.favorite { "★ " } else { "  " };

            let content = format!("{}{}", favorite_marker, service.as_str());

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if service.favorite {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[0]);

    // Draw help text at bottom
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓/j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Select  "),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle ★  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Close"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(help, chunks[1]);
}

fn draw_detail_popup(f: &mut Frame, app: &App) {
    // Calculate popup size and position (centered, larger)
    let area = centered_rect(70, 70, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the popup container
    let title = if app.detail_loading {
        "Loading Details..."
    } else if app.selected_index < app.items.len() {
        "Resource Details"
    } else {
        "Details"
    };

    let popup_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    f.render_widget(popup_block, area);

    // Create inner area for content
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split inner area for list and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner_area);

    // Create detail items with key-value formatting
    let items: Vec<ListItem> = app
        .detail_content
        .iter()
        .map(|(key, value)| {
            let content = if value.is_empty() {
                Line::from(vec![
                    Span::styled(key, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(value, Style::default().fg(Color::White)),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[0]);

    // Draw help text at bottom
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓/j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Scroll  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" or "),
            Span::styled("i", Style::default().fg(Color::Yellow)),
            Span::raw(": Close"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(help, chunks[1]);
}

fn draw_quit_confirmation(f: &mut Frame) {
    // Calculate popup size and position (small, centered)
    let area = centered_rect(40, 20, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the popup container
    let popup_block = Block::default()
        .title("Confirm Quit")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    f.render_widget(popup_block, area);

    // Create inner area for content
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split inner area for message and buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner_area);

    // Message
    let message = Paragraph::new("Are you sure you want to quit?")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    f.render_widget(message, chunks[0]);

    // Buttons
    let buttons = Line::from(vec![
        Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("es  "),
        Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("o"),
    ]);
    let buttons_widget = Paragraph::new(buttons)
        .alignment(Alignment::Center);
    f.render_widget(buttons_widget, chunks[2]);
}

// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
