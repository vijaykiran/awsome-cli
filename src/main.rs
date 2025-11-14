use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod app;
mod ui;
mod aws;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Initialize AWS client
    let _ = app.initialize_aws_client().await;

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle popup-specific controls first
                if app.show_service_popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char(' ') => {
                            app.toggle_service_popup();
                        }
                        KeyCode::Down => app.popup_next(),
                        KeyCode::Up => app.popup_previous(),
                        KeyCode::Enter => app.select_popup_service(),
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            app.toggle_favorite();
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                } else {
                    // Handle main view controls
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(' ') => app.toggle_service_popup(),
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            app.refresh_resources().await?;
                        }
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.previous_item(),
                        KeyCode::Enter => app.select_item().await?,
                        _ => {}
                    }
                }
            }
        }
    }
}
