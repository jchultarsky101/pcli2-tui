use crate::app::{App, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Define the main layout - without the top bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(10),   // Main content area (now starts at the top)
                Constraint::Length(6), // Multi-line log window
                Constraint::Length(1), // Contextual key bindings line
            ]
            .as_ref(),
        )
        .split(f.area());

    // Draw the main content area based on current state (this starts at the top now)
    draw_main_content(f, main_chunks[0], app);

    // Draw the status bar
    draw_status_bar(f, main_chunks[1], app);

    // Draw contextual key bindings at the bottom of the screen
    draw_contextual_key_bindings(f, app, main_chunks[2]);

    // Draw search modal if active
    if app.show_search_modal {
        draw_search_modal(f, f.area(), app);
    }

    // Draw help modal if active
    if matches!(app.current_state, AppState::Help) {
        draw_help_modal(f, f.area(), app);
    }

    // Draw geometric match modal if active
    if app.show_geometric_match_modal {
        draw_geometric_match_modal(f, f.area(), app);
    }
}


fn draw_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_state {
        AppState::Folders | AppState::Assets => draw_folder_asset_view(f, area, app),
        AppState::Search => draw_search_view(f, area, app),
        AppState::Uploading | AppState::Downloading => draw_upload_download_view(f, area, app),
        AppState::Help => draw_folder_asset_view(f, area, app), // Show folder/asset view underneath help modal
        AppState::CommandHistory => draw_command_history_view(f, area, app),
        AppState::Log => draw_log_view(f, area, app),
        AppState::PaneResize => draw_folder_asset_view(f, area, app), // Use the same view but indicate resize mode
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
        Color::Rgb(255, 215, 0)  // Gold color for active pane (consistent with other panes)
    } else {
        Color::Rgb(100, 100, 100)  // Muted gray for inactive
    };
    let title = format!(
        " 📁 Folder(s) [{}] ",
        app.current_folder.as_deref().unwrap_or("/")
    );
    let items: Vec<ListItem> = app
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            let is_selected = i == app.selected_folder_index;

            let content = if folder.uuid == ".." {
                let special_style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(106, 90, 205))  // Indigo for parent folder
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default()
                        .fg(Color::Rgb(173, 216, 230))  // Light blue for parent folder
                        .add_modifier(Modifier::ITALIC)
                };
                Line::from(vec![Span::styled(
                    format!("🔙 {}", folder.name),
                    special_style,
                )])
            } else {
                // Create spans for folder name and stats separately
                let name_span = Span::styled(
                    format!("📂 {}", folder.name),
                    if is_selected {
                        Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)  // Forest green bg with white text when selected (same as assets)
                    } else {
                        Style::default().fg(Color::Rgb(255, 215, 0))  // Gold text for folder name (same as assets)
                    }
                );

                let stats_span = Span::styled(
                    format!(" ({} 📁, {} 📎)", folder.folders_count, folder.assets_count),
                    if is_selected {
                        Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::Rgb(200, 200, 200))  // Lighter gray stats when selected
                    } else {
                        Style::default().fg(Color::Rgb(150, 150, 150))  // Subdued gray for stats
                    }
                );

                Line::from(vec![name_span, stats_span])
            };

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White));  // Forest green highlight (same as assets)

    f.render_widget(list, area);
}

fn draw_assets_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let is_active = matches!(app.active_pane, crate::app::ActivePane::Assets);
    let border_color = if is_active {
        Color::Rgb(255, 215, 0)  // Gold color for active pane (consistent with other panes)
    } else {
        Color::Rgb(100, 100, 100)  // Muted gray for inactive
    };

    // Determine the title based on whether we're loading assets for selection
    let title = if app.assets_loading_for_selection {
        " 📎 Assets - Loading... ".to_string()
    } else {
        " 📎 Asset(s) ".to_string()
    };

    let items: Vec<ListItem> = if app.assets_loading_for_selection {
        // Show a loading indicator
        vec![ListItem::new(Line::from(Span::styled(
            "⏳ Loading assets...",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))  // Cornflower blue
                .add_modifier(Modifier::ITALIC),
        )))]
    } else {
        app.assets
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let is_selected = i == app.selected_asset_index;
                let style = if is_selected {
                    Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)  // Forest green for selection
                } else {
                    Style::default().fg(Color::Rgb(255, 215, 0))  // Gold for unselected
                };

                let icon = match asset.file_type.as_str() {
                    "model" => "🏗️",    // Building/construction icon for 3D models
                    "document" => "📝", // Document icon
                    "image" => "🖼️",    // Image icon
                    "video" => "🎥",    // Video icon
                    "audio" => "🎧",    // Audio icon
                    "archive" => "📦",  // Archive icon
                    "code" => "💻",     // Code/icon
                    _ => "📄",          // Default document icon
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
                .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White));  // Forest green highlight

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


fn draw_help_modal(f: &mut Frame, area: Rect, _app: &App) {
    // Create a centered modal window
    let popup_area = centered_rect(60, 80, area);

    // Clear the background first
    f.render_widget(Clear, popup_area);

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
        Line::from("  Tab            - Switch between panes (forward)"),
        Line::from("  Shift+Tab      - Switch between panes (reverse)"),
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
        Line::from("  g              - Perform geometric match on selected asset (in Assets view)"),
        Line::from(""),
        Line::from("Mode Switching:"),
        Line::from("  u              - Upload mode"),
        Line::from("  d              - Download mode"),
        Line::from(""),
        Line::from("Search Dialog:"),
        Line::from("  /              - Open search dialog"),
        Line::from("  Tab            - Switch focus in search dialog (forward)"),
        Line::from("  Shift+Tab      - Switch focus in search dialog (reverse)"),
        Line::from("  Enter          - Perform search or close search results"),
        Line::from("  Esc            - Close search dialog"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  Ctrl+N         - Enter pane resize mode"),
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
                .title(" 💡 Help ")  // Changed title with padding spaces and emoji
                .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))  // Gold border
                .padding(ratatui::widgets::Padding::uniform(1))  // Add 1 space padding on all sides
                .style(Style::default().bg(Color::Rgb(40, 40, 50))),  // Dark blue-gray background
        )
        .style(Style::default().fg(Color::Rgb(220, 220, 220)))  // Light gray text for better readability
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}

// Helper function to create a centered rect

fn draw_contextual_key_bindings(f: &mut Frame, app: &App, area: Rect) {
    // Define key bindings based on current state
    let key_bindings_text = match app.current_state {
        crate::app::AppState::Folders => {
            "tab:switch | j/k:nav | enter:sel | /:search | h:help | q:quit"
        }
        crate::app::AppState::Assets => {
            "tab:switch | j/k:nav | enter:sel | g:geom-match | /:search | h:help | q:quit"
        }
        crate::app::AppState::Search => {
            "enter:search | esc:cancel | ↑↓:nav | d:download | q:quit"
        }
        crate::app::AppState::Uploading | crate::app::AppState::Downloading => "q:quit",
        crate::app::AppState::Help => "q/esc:close",
        crate::app::AppState::CommandHistory => "q/esc:close",
        crate::app::AppState::Log => "↑↓:scroll | q:quit",
        crate::app::AppState::PaneResize => "↑↓←→:resize | enter:ok | esc/q:cancel",
    };

    let key_bindings_paragraph = Paragraph::new(ratatui::text::Line::from(key_bindings_text))
        .style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Rgb(220, 220, 220))  // Light gray text
                .bg(ratatui::style::Color::Rgb(60, 60, 60)),   // Darker background
        );

    f.render_widget(key_bindings_paragraph, area);
}
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
    let start_idx = if app.log_entries.len() < 7 {
        // If we have fewer than 7 entries, show from the beginning
        0
    } else {
        // Otherwise, use the scroll position logic
        app.log_scroll_position.saturating_sub(3) // Show 3 entries before current position to allow for 7 total
    };
    let end_idx = std::cmp::min(start_idx + 7, app.log_entries.len()); // Show up to 7 entries to fill the pane

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
                        ratatui::text::Span::styled(
                            "✓ ",
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Green)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            parts[1].trim_start(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                        ),
                    ])
                } else {
                    ratatui::text::Line::from(entry.as_str())
                }
            } else if entry.contains("✗ ERROR:") {
                // Error entry - red color
                let parts: Vec<&str> = entry.splitn(2, "✗ ERROR:").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            "✗ ",
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Red)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            parts[1].trim_start(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Red),
                        ),
                    ])
                } else {
                    ratatui::text::Line::from(entry.as_str())
                }
            } else if entry.contains("✓ CACHED:") {
                // Cached entry - yellow color with cache icon
                let parts: Vec<&str> = entry.splitn(2, "✓ CACHED:").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            "🗂️ ", // Cache icon
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Yellow)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            parts[1].trim_start(),
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Yellow)
                                .bg(ratatui::style::Color::DarkGray),
                        ),
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
            ratatui::text::Line::from(format!(
                "Status: {} | Path: {}",
                app.status_message,
                app.current_folder.as_deref().unwrap_or("/")
            )),
            ratatui::text::Line::from(format!("Last Cmd: {}", app.last_executed_command)),
            ratatui::text::Line::from(match app.current_state {
                AppState::Folders => {
                    "Folders View (j/k: nav, Enter: open, a: assets, /: search, h: help, c: cmd history, l: log, Tab/Shift+Tab: switch pane, F10: menu, Backspace: back, q: quit)"
                }
                AppState::Assets => {
                    "Assets View (j/k: nav, d: download, h: help, c: cmd history, l: log, Tab/Shift+Tab: switch pane, F10: menu, Backspace: back, q: quit)"
                }
                AppState::Search => {
                    "Search Mode (type and Enter: search, h: help, c: cmd history, l: log, Tab/Shift+Tab: switch pane, F10: menu, Esc: cancel, q: quit)"
                }
                AppState::Uploading => {
                    "Upload Mode (u: upload, h: help, c: cmd history, l: log, q: quit)"
                }
                AppState::Downloading => {
                    "Download Mode (select and d: download, h: help, c: cmd history, l: log, q: quit)"
                }
                AppState::Help => "Help Screen (q/Esc: close help)",
                AppState::CommandHistory => "Command History (q/Esc: close)",
                AppState::Log => "Log View (Arrow keys: scroll, q/Esc: close)",
                AppState::PaneResize => {
                    "Pane Resize Mode (↑↓←→: resize, Enter: apply, Esc/q: cancel)"
                }
            }),
            ratatui::text::Line::from(match app.current_state {
                AppState::Log => "↑/↓: scroll | q/Esc: exit | F10: menu | Ctrl+N: resize",
                AppState::Search => "Enter: search | Esc: cancel | F10: menu | Ctrl+N: resize",
                AppState::PaneResize => {
                    "↑/↓/←/→: resize | Enter: apply | Esc/q: cancel | F10: exit"
                }
                _ => {
                    // Default help for main browsing modes
                    "Tab: switch panes | Shift+Tab: reverse switch | F10: menu | j/k: nav | Enter: select | h: help | Ctrl+N: resize | q: quit"
                }
            }),
        ]
    } else {
        log_lines
    };

    // Determine the border color based on whether this pane is active
    let border_color = if matches!(app.active_pane, crate::app::ActivePane::Log) {
        Color::Rgb(255, 215, 0)  // Gold color for active pane (consistent with other panes)
    } else {
        Color::Rgb(80, 80, 80)   // Darker gray for inactive
    };

    let list = ratatui::widgets::List::new(list_items)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(format!(
                    " 📝 Log [{}/{}] ", // Added log emoji
                    app.log_scroll_position + 1,
                    app.log_entries.len()
                ))
                .border_style(ratatui::style::Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        )
        .style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Rgb(200, 200, 200)),  // Same text color as other panes
        )
        .highlight_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(70, 130, 180))  // Steel blue highlight
                .fg(ratatui::style::Color::White),
        );

    f.render_widget(list, area);
}

fn draw_command_history_view(f: &mut Frame, area: Rect, app: &App) {
    let title = " 📋 Command History ";
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
                .title(title)
                .border_style(ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Rgb(147, 112, 219))  // Medium purple
                    .add_modifier(ratatui::style::Modifier::BOLD)),
        )
        .highlight_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(147, 112, 219))  // Medium purple
                .fg(ratatui::style::Color::White),
        );

    f.render_widget(list, area);
}

fn draw_log_view(f: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        " 📝 Log [{}/{}] ",
        app.log_scroll_position + 1,
        app.log_entries.len()
    );

    // Show a portion of the log entries based on scroll position
    let start_idx = app.log_scroll_position.saturating_sub(10); // Show 10 entries before current position
    let end_idx = std::cmp::min(start_idx + 20, app.log_entries.len()); // Show 20 entries total

    // Create list items with highlighting for the selected item
    let list_items: Vec<ratatui::widgets::ListItem> = app
        .log_entries
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .enumerate()
        .map(|(idx, entry)| {
            // Check if this item corresponds to the current scroll position
            let is_selected = start_idx + idx == app.log_scroll_position;

            if is_selected {
                // Style for selected item - use a more prominent highlight
                ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        "▶ ",
                        ratatui::style::Style::default()
                            .bg(ratatui::style::Color::Rgb(70, 130, 180))  // Steel blue
                            .fg(ratatui::style::Color::Rgb(255, 215, 0))   // Gold
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    ratatui::text::Span::styled(
                        entry.as_str(),
                        ratatui::style::Style::default()
                            .bg(ratatui::style::Color::Rgb(70, 130, 180))  // Steel blue
                            .fg(ratatui::style::Color::White)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]))
            } else {
                // Style for non-selected items
                ratatui::widgets::ListItem::new(ratatui::text::Line::from(entry.as_str()))
            }
        })
        .collect();

    let list = ratatui::widgets::List::new(list_items).block(
        ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .title(title)
            .border_style(ratatui::style::Style::default()
                .fg(ratatui::style::Color::Rgb(100, 149, 237))  // Cornflower blue border
                .add_modifier(ratatui::style::Modifier::BOLD)),
    )
    .style(
        ratatui::style::Style::default()
            .bg(ratatui::style::Color::Rgb(30, 30, 30))  // Same background as other panes
            .fg(ratatui::style::Color::Rgb(200, 200, 200)),  // Same text color as other panes
    );

    f.render_widget(list, area);
}

fn draw_search_modal(f: &mut Frame, area: Rect, app: &App) {
    // Create a centered modal window
    let popup_area = centered_rect(60, 40, area);

    // Clear the background first
    f.render_widget(Clear, popup_area);

    // Draw outer frame for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))  // Gold border to match other panes
        .title(" 🔍 Search ")  // Added spaces for padding
        .style(Style::default().bg(Color::Rgb(30, 30, 40))); // Slightly different dark background

    f.render_widget(modal_block, popup_area);

    // Divide the modal into sections for input and results (accounting for the border)
    let inner_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input section
            Constraint::Min(1),    // Results section
        ])
        .split(inner_area);

    // Input section - now just the input field without a label
    // Draw the search input field with proper alignment and enhanced visual cues
    let input_border_color = if matches!(app.search_modal_focus, crate::app::SearchModalFocus::Input) {
        Color::Yellow // Highlight with yellow when focused (consistent with other panes)
    } else {
        Color::Gray // More visible color when not focused
    };

    let input_field = Paragraph::new(format!("{}█", app.search_input_buffer)) // Add a visual cursor
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(input_border_color).add_modifier(Modifier::BOLD)) // Highlight when focused
                .style(Style::default().bg(Color::Rgb(40, 40, 40))), // Slightly lighter background
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(input_field, chunks[0]); // Use the whole input section for the field

    // Results section
    let results_title = format!(" Results ({}) ", app.search_results.len()); // Renamed to "Results" and padded with spaces

    let results_list_items = if app.command_in_progress {
        // Show a searching indicator when command is in progress
        vec![ListItem::new(
            Line::from(Span::styled(
                "Searching...",
                Style::default().fg(Color::Yellow)
            ))
        )]
    } else if app.search_results.is_empty() {
        // Show a message when there are no search results
        vec![ListItem::new(
            Line::from(Span::styled(
                "No results found",
                Style::default().fg(Color::DarkGray)
            ))
        )]
    } else {
        app.search_results
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let is_selected = i == app.selected_search_result_index;
                let style = if is_selected {
                    Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)  // Forest green to match other selections
                } else {
                    Style::default().fg(Color::Rgb(255, 255, 0))  // Gold to match other unselected items
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
            .collect::<Vec<ListItem>>()
    };

    // Determine border color based on focus state
    let results_border_color = if matches!(app.search_modal_focus, crate::app::SearchModalFocus::Results) {
        Color::Rgb(255, 215, 0) // Gold/yellow when focused (to match search input field)
    } else {
        Color::Rgb(100, 100, 100) // More visible color when not focused
    };

    let results_list = List::new(results_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(results_border_color).add_modifier(Modifier::BOLD)) // Highlight when focused
                .title(results_title)
        ) // Consistent border styling
        .highlight_style(Style::default().bg(Color::Rgb(135, 206, 235)).fg(Color::Black)); // Light sky blue for contrast

    // Render the results list
    f.render_widget(results_list, chunks[1]);
}

fn draw_geometric_match_modal(f: &mut Frame, area: Rect, app: &App) {
    // Create a centered modal window
    let popup_area = centered_rect(60, 40, area);

    // Clear the background first
    f.render_widget(Clear, popup_area);

    // Draw outer frame for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))  // Gold border
        .title(" 🔍 Geometric Match Results ")  // Added spaces for padding
        .style(Style::default().bg(Color::Rgb(30, 30, 40))); // Dark background matching theme

    f.render_widget(modal_block, popup_area);

    // Divide the modal into sections for results
    let inner_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    // Results section
    let results_title = format!(" Results ({}) ", app.geometric_match_results.len()); // Renamed to "Results" and padded with spaces

    let results_list_items = if app.command_in_progress {
        // Show a searching indicator when command is in progress
        vec![ListItem::new(
            Line::from(Span::styled(
                "Processing geometric match...",
                Style::default().fg(Color::Yellow)
            ))
        )]
    } else if app.geometric_match_results.is_empty() {
        // Show a message when there are no geometric match results
        vec![ListItem::new(
            Line::from(Span::styled(
                "No geometric matches found",
                Style::default().fg(Color::DarkGray)
            ))
        )]
    } else {
        app.geometric_match_results
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let is_selected = i == app.selected_asset_index; // Using selected_asset_index for now
                let style = if is_selected {
                    Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White) // Forest green to match other selections
                } else {
                    Style::default().fg(Color::Rgb(255, 255, 0)) // Gold to match other unselected items
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
            .collect::<Vec<ListItem>>()
    };

    let results_list = List::new(results_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)) // Gold border
                .title(results_title)
        ) // Consistent border styling
        .highlight_style(Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)); // Forest green highlight

    // Render the results list
    f.render_widget(results_list, inner_area);
}
