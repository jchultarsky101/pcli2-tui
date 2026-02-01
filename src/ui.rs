use crate::app::{App, AppState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    widgets::Clear,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Define the main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Top bar (menu)
                Constraint::Min(10),    // Main content area
                Constraint::Length(7),  // Multi-line log window (increased height)
            ]
            .as_ref(),
        )
        .split(f.area());

    // Draw the top menu bar
    draw_top_bar(f, chunks[0], app);

    // Draw the main content area based on current state
    draw_main_content(f, chunks[1], app);

    // Draw the status bar
    draw_status_bar(f, chunks[2], app);

    // Draw search modal if active
    if app.show_search_modal {
        draw_search_modal(f, f.area(), app);
    }
}

fn draw_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let menu_items = [
        "File",
        "Edit",
        "View",
        "Search",
        "Help"
    ];

    let mut spans = Vec::new();

    // Add spinner if command is in progress
    if app.command_in_progress {
        // Create a simple spinner animation based on time
        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let frame_index = (std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() / 100) as usize % frames.len();
        spans.push(Span::styled(format!("{} ", frames[frame_index]), Style::default().fg(Color::Yellow)));
    }

    for (i, item) in menu_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }

        let style = if matches!(app.current_state, AppState::Search) && *item == "Search" {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        spans.push(Span::styled(*item, style));
    }

    // Determine the border color based on whether this pane is active
    let is_active = matches!(app.active_pane, crate::app::ActivePane::Log);
    let bg_color = if is_active { Color::DarkGray } else { Color::Blue }; // Different background when active
    let fg_color = if is_active { Color::Yellow } else { Color::White }; // Different text color when active

    // Apply the active state styling to the menu items
    let mut styled_spans = Vec::new();
    for (i, span) in spans.iter().enumerate() {
        if i == 0 && span.content.chars().any(|c| ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'].contains(&c)) {
            // If this is the spinner, keep its color
            styled_spans.push(span.clone());
        } else {
            styled_spans.push(Span::styled(span.content.clone(), Style::default().fg(fg_color)));
        }
    }

    let title = Block::default()
        .title(Line::from(styled_spans))
        .borders(Borders::NONE) // Remove borders
        .style(Style::default().add_modifier(Modifier::BOLD).bg(bg_color));

    f.render_widget(title, area);
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_state {
        AppState::Folders | AppState::Assets => draw_folder_asset_view(f, area, app),
        AppState::Search => draw_search_view(f, area, app),
        AppState::Uploading | AppState::Downloading => draw_upload_download_view(f, area, app),
        AppState::Help => draw_help_view(f, area, app),
        AppState::CommandHistory => draw_command_history_view(f, area, app),
        AppState::Log => draw_log_view(f, area, app),
    }
}

fn draw_folder_asset_view(f: &mut Frame, area: Rect, app: &mut App) {
    // Split the main area into left (folders) and right (assets) panels
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    // Draw folders on the left
    draw_folders_panel(f, horizontal_chunks[0], app);

    // Draw assets on the right
    draw_assets_panel(f, horizontal_chunks[1], app);
}

fn draw_folders_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let is_active = matches!(app.active_pane, crate::app::ActivePane::Folders);
    let border_color = if is_active {
        Color::Yellow
    } else {
        Color::Gray
    };
    let title = format!("Folders - {}", app.current_folder.as_deref().unwrap_or("/"));
    let items: Vec<ListItem> = app
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            let is_selected = i == app.selected_folder_index;
            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = if folder.uuid == ".." {
                let special_style = if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::ITALIC)
                };
                Line::from(vec![Span::styled(
                    format!("📁 {}", folder.name),
                    special_style,
                )])
            } else {
                Line::from(vec![Span::styled(
                    format!(
                        "📁 {} ({} folders, {} assets)",
                        folder.name, folder.folders_count, folder.assets_count
                    ),
                    style,
                )])
            };

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(list, area);
}

fn draw_assets_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let is_active = matches!(app.active_pane, crate::app::ActivePane::Assets);
    let border_color = if is_active {
        Color::Yellow
    } else {
        Color::Gray
    };

    // Determine the title based on whether we're loading assets for selection
    let title = if app.assets_loading_for_selection {
        "Assets - Loading...".to_string()
    } else {
        format!("Assets - {}", app.current_folder.as_deref().unwrap_or("/"))
    };

    let items: Vec<ListItem> = if app.assets_loading_for_selection {
        // Show a loading indicator
        vec![ListItem::new(Line::from(Span::styled(
            "⏳ Loading assets...",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::ITALIC),
        )))]
    } else {
        app.assets
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let is_selected = i == app.selected_asset_index;
                let style = if is_selected {
                    Style::default().bg(Color::Green).fg(Color::White)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                let icon = match asset.file_type.as_str() {
                    "model" => "🏗️",    // Building/construction icon for 3D models
                    "document" => "📄", // Document icon
                    "image" => "🖼️",    // Image icon
                    "video" => "🎬",    // Video icon
                    "audio" => "🎵",    // Audio icon
                    "archive" => "📦",  // Archive icon
                    "code" => "💻",     // Code/icon
                    _ => "📁",          // Default folder icon
                };

                let content = Line::from(vec![Span::styled(
                    format!("{} {}", icon, asset.name),
                    style,
                )]);

                ListItem::new(content)
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(Style::default().bg(Color::Green).fg(Color::White));

    f.render_widget(list, area);
}

fn draw_search_view(f: &mut Frame, area: Rect, app: &App) {
    // Show search input with instructions
    let search_block = Block::default().borders(Borders::ALL).title(format!(
        "Search: {} [Press Enter to search, Esc to cancel]",
        app.search_query
    ));

    f.render_widget(search_block, area);
}

fn draw_upload_download_view(f: &mut Frame, area: Rect, app: &App) {
    let title = match app.current_state {
        AppState::Uploading => "Upload Mode",
        AppState::Downloading => "Download Mode",
        _ => "", // This shouldn't happen
    };

    let text = match app.current_state {
        AppState::Uploading => {
            vec![
                Line::from("Upload Mode Active"),
                Line::from("Press 'u' to select a file to upload"),
                Line::from("Press 'q' to return to main view"),
            ]
        }
        AppState::Downloading => {
            vec![
                Line::from("Download Mode Active"),
                Line::from("Select an asset and press 'd' to download"),
                Line::from("Press 'q' to return to main view"),
            ]
        }
        _ => vec![Line::from("Unknown mode")],
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(paragraph, area);
}

fn draw_help_view(f: &mut Frame, area: Rect, _app: &App) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "PCLI2-TUI Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j / Down Arrow - Move down in current pane"),
        Line::from("  k / Up Arrow   - Move up in current pane"),
        Line::from("  Tab            - Switch between panes"),
        Line::from("  Enter          - Open selected folder or perform action on asset"),
        Line::from("  Backspace      - Go back to parent folder"),
        Line::from(""),
        Line::from("View Controls:"),
        Line::from("  a              - Switch to assets view"),
        Line::from("  h              - Show this help screen"),
        Line::from("  /              - Enter search mode"),
        Line::from(""),
        Line::from("Asset Operations:"),
        Line::from("  d              - Download selected asset (in Assets view)"),
        Line::from(""),
        Line::from("Mode Switching:"),
        Line::from("  u              - Upload mode"),
        Line::from("  d              - Download mode"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  q / Ctrl+C     - Quit application"),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' or 'Esc' to close this help screen",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help - PCLI2-TUI")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    // Calculate centered area for the help box
    let popup_area = centered_rect(60, 80, area);
    f.render_widget(paragraph, popup_area);
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

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    // Show a portion of the log entries based on scroll position
    let start_idx = app.log_scroll_position.saturating_sub(2); // Show 2 entries before current position
    let end_idx = std::cmp::min(start_idx + 5, app.log_entries.len()); // Show 5 entries total

    let log_lines: Vec<ratatui::text::Line> = app
        .log_entries
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|entry| {
            // Check if the entry contains success or error indicators
            if entry.contains("✓ SUCCESS:") {
                // Success entry - green color
                let parts: Vec<&str> = entry.splitn(2, "✓ SUCCESS:").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("✓ ",
                            ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(ratatui::style::Modifier::BOLD)),
                        ratatui::text::Span::styled(parts[1].trim_start(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Green)),
                    ])
                } else {
                    ratatui::text::Line::from(entry.as_str())
                }
            } else if entry.contains("✗ ERROR:") {
                // Error entry - red color
                let parts: Vec<&str> = entry.splitn(2, "✗ ERROR:").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("✗ ",
                            ratatui::style::Style::default().fg(ratatui::style::Color::Red).add_modifier(ratatui::style::Modifier::BOLD)),
                        ratatui::text::Span::styled(parts[1].trim_start(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Red)),
                    ])
                } else {
                    ratatui::text::Line::from(entry.as_str())
                }
            } else if entry.contains("✓ CACHED:") {
                // Cached entry - yellow color
                let parts: Vec<&str> = entry.splitn(2, "✓ CACHED:").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("✓ ",
                            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)),
                        ratatui::text::Span::styled(parts[1].trim_start(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)),
                    ])
                } else {
                    ratatui::text::Line::from(entry.as_str())
                }
            } else {
                // Regular entry - white color
                ratatui::text::Line::from(entry.as_str())
            }
        })
        .collect();

    // If no log entries, show status information
    let list_items = if log_lines.is_empty() {
        vec![
            ratatui::text::Line::from(format!("Status: {}", app.status_message)),
            ratatui::text::Line::from(format!("Current Path: {}", app.current_folder.as_deref().unwrap_or("/"))),
            ratatui::text::Line::from(format!("Last Cmd: {}", app.last_executed_command)),
            ratatui::text::Line::from(match app.current_state {
                AppState::Folders =>
                    "Folders View (j/k: nav, Enter: open, a: assets, /: search, h: help, c: cmd history, l: log, Tab: switch pane, Backspace: back, q: quit)",
                AppState::Assets =>
                    "Assets View (j/k: nav, d: download, h: help, c: cmd history, l: log, Tab: switch pane, Backspace: back, q: quit)",
                AppState::Search =>
                    "Search Mode (type and Enter: search, h: help, c: cmd history, l: log, Tab: switch pane, Esc: cancel, q: quit)",
                AppState::Uploading => "Upload Mode (u: upload, h: help, c: cmd history, l: log, q: quit)",
                AppState::Downloading => "Download Mode (select and d: download, h: help, c: cmd history, l: log, q: quit)",
                AppState::Help => "Help Screen (q/Esc: close help)",
                AppState::CommandHistory => "Command History (q/Esc: close)",
                AppState::Log => "Log View (Arrow keys: scroll, q/Esc: close)",
            }),
        ]
    } else {
        log_lines
    };

    // Determine the border color based on whether this pane is active
    let border_color = if matches!(app.active_pane, crate::app::ActivePane::Log) { Color::Yellow } else { Color::Gray };

    let list = ratatui::widgets::List::new(list_items)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(format!("Log (Scroll: {}/{})",
                    app.log_scroll_position + 1,
                    app.log_entries.len()))
                .border_style(ratatui::style::Style::default().fg(border_color)),
        )
        .style(ratatui::style::Style::default().bg(ratatui::style::Color::DarkGray).fg(ratatui::style::Color::White))
        .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));

    f.render_widget(list, area);
}

fn draw_command_history_view(f: &mut Frame, area: Rect, app: &App) {
    let title = "Command History";
    let commands: Vec<ratatui::text::Line> = app
        .command_history
        .iter()
        .rev() // Show most recent first
        .take(50) // Limit to last 50 commands
        .map(|cmd| ratatui::text::Line::from(cmd.as_str()))
        .collect();

    let list = ratatui::widgets::List::new(commands)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(title),
        )
        .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));

    f.render_widget(list, area);
}

fn draw_log_view(f: &mut Frame, area: Rect, app: &App) {
    let title = format!("Log (Scroll: {}/{})",
        app.log_scroll_position + 1,
        app.log_entries.len()
    );

    // Show a portion of the log entries based on scroll position
    let start_idx = app.log_scroll_position.saturating_sub(10); // Show 10 entries before current position
    let end_idx = std::cmp::min(start_idx + 20, app.log_entries.len()); // Show 20 entries total

    let log_lines: Vec<ratatui::text::Line> = app
        .log_entries
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|entry| ratatui::text::Line::from(entry.as_str()))
        .collect();

    let list = ratatui::widgets::List::new(log_lines)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(title),
        )
        .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));

    f.render_widget(list, area);
}

fn draw_search_modal(f: &mut Frame, area: Rect, app: &App) {
    // Create a centered modal window
    let popup_area = centered_rect(40, 15, area);
    
    // Create the input field content
    let input_content = format!("Search: {}", app.search_input_buffer);
    
    // Create the modal content
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("Search Assets", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(input_content),
        Line::from(""),
        Line::from(Span::styled("Press Enter to search, Esc to cancel", Style::default().add_modifier(Modifier::ITALIC))),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .style(Style::default().bg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(Clear, popup_area); // Clear the background
    f.render_widget(paragraph, popup_area);
}

// Helper function to create a centered rect
