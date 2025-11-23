use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;

pub mod app;
pub mod aws;
pub mod ui;

use app::App;

// Re-export run_app so it can be used by main.rs
pub async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Handle popup-specific controls first
            if app.show_quit_confirm {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(()),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.hide_quit_confirmation();
                    }
                    _ => {}
                }
            } else if app.show_detail_popup {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('I') => {
                        app.close_detail_popup();
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.detail_scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.detail_scroll_up(),
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.show_quit_confirmation();
                    }
                    _ => {}
                }
            } else if app.show_service_popup {
                match key.code {
                    KeyCode::Esc | KeyCode::Char(' ') => {
                        app.toggle_service_popup();
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.popup_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.popup_previous(),
                    KeyCode::Enter => app.select_popup_service(),
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        app.toggle_favorite();
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.show_quit_confirmation();
                    }
                    _ => {}
                }
            } else {
                // Handle main view controls
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.show_quit_confirmation();
                    }
                    KeyCode::Char(' ') => app.toggle_service_popup(),
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        app.show_resource_details().await?;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app.refresh_resources().await?;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                    KeyCode::Enter => app.select_item().await?,
                    _ => {}
                }
            }
        }

        // Update animation frame if loading
        if app.is_loading() {
            app.tick_animation();
        }
    }
}
