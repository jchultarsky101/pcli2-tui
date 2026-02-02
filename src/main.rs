use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod pcli_commands;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> Result<()> {
    // Load initial folder data
    app.load_folders_for_current_context().await;

    // Pre-fetch assets for the first folder if available
    if !app.folders.is_empty() {
        // Select the first folder (skip parent indicator if present)
        if app.folders[0].uuid == ".." && app.folders.len() > 1 {
            app.selected_folder_index = 1;
        } else {
            app.selected_folder_index = 0;
        }

        // Load assets for the selected folder
        app.load_assets_for_selected_folder().await;
    }

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }

                app.handle_key_event(key).await;
            } else if let Event::Mouse(mouse) = event::read()? {
                app.handle_mouse_event(mouse).await;
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
