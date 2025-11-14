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

    // Draw popup on top if active
    if app.show_service_popup {
        draw_service_popup(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let favorites = app.get_favorite_services();

    let mut spans = vec![
        Span::styled("AWSOME ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| "),
    ];

    if favorites.is_empty() {
        spans.push(Span::styled("No favorites - Press Space to select service", Style::default().fg(Color::Gray)));
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

            spans.push(Span::styled(
                format!(" {} ", service.short_name()),
                style,
            ));

            if idx < favorites.len() - 1 {
                spans.push(Span::raw("• "));
            }
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled("[Space: More]", Style::default().fg(Color::DarkGray)));
    }

    let header = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
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
            let style = if i == app.selected_index {
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

    let status_text = Span::styled(
        app.status_message.as_str(),
        Style::default().fg(status_color),
    );

    let footer = Paragraph::new(Line::from(vec![status_text]))
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
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
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
