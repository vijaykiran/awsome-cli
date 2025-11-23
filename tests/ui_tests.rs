use awsome::app::App;
use awsome::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn test_ui_initial_state() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    // Draw the UI
    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    // Assert on the buffer content
    let buffer = terminal.backend().buffer();

    // Check for title
    // Note: The actual title format depends on ui.rs implementation.
    // Based on typical TUI apps, we expect some header or content.
    // Let's check for "AWS Resource Manager" if that's the title, or service names.

    // We know the app starts with "Initializing AWS client..." in the list
    // and "Press Space for services..." in the status bar.

    // Let's check for the presence of these strings in the buffer cells
    let content = buffer_to_string(buffer);

    assert!(content.contains("Initializing AWS client..."));
    assert!(content.contains("Press Space for services"));
    assert!(content.contains("EC2 Instances")); // Active service tab
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            s.push_str(buffer[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}
