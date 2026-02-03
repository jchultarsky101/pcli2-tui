use crate::app::{App, AppState, Asset};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    widgets::{Cell, Row, Table},
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

    // Draw asset details modal if active
    if app.show_asset_details_modal {
        draw_asset_details_modal(f, f.area(), app);
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

    let title = if app.assets_loading_for_selection {
        " 📎 Assets - Loading... ".to_string()
    } else {
        " 📎 Asset(s) ".to_string()
    };

    // Extract all unique metadata keys from assets
    let mut all_metadata_keys = std::collections::HashSet::<String>::new();
    for asset in &app.assets {
        if let Some(obj) = asset.metadata.as_object() {
            for key in obj.keys() {
                // Special handling for the case where metadata contains a "meta" key that wraps actual metadata
                if key == "meta" {
                    // If the "meta" key contains an object, extract keys from that object instead
                    if let Some(meta_obj) = obj.get(key).and_then(|v| v.as_object()) {
                        for meta_key in meta_obj.keys() {
                            all_metadata_keys.insert(meta_key.clone());
                        }
                    } else {
                        // If "meta" doesn't contain an object, use it as a regular key
                        all_metadata_keys.insert(key.clone());
                    }
                } else {
                    // Regular handling for other keys
                    all_metadata_keys.insert(key.clone());
                }
            }
        }
    }

    // Convert to sorted vector
    let mut sorted_metadata_keys: Vec<String> = all_metadata_keys.into_iter().collect();
    sorted_metadata_keys.sort();

    // Define headers for the table
    let mut headers = vec!["", "Name", "Path"]; // Icon, Name, Path (removed Type column)
    for key in &sorted_metadata_keys {
        headers.push(key.as_str());
    }

    // Calculate optimal column widths based on content
    let column_widths = if app.assets.is_empty() {
        // Default widths when no assets
        let mut widths = vec![
            Constraint::Length(3),  // Icon column (single character + padding)
            Constraint::Min(15),    // Name column (minimum width for readability)
            Constraint::Min(15),    // Path column (minimum width for readability)
        ];

        // Add constraints for metadata columns
        for _ in &sorted_metadata_keys {
            widths.push(Constraint::Min(10)); // Minimum width for metadata columns
        }
        widths
    } else {
        // Calculate max lengths for each column based on content
        let max_icon_len = 1; // Icons are single characters (don't need mut)
        let mut max_name_len = "Name".len(); // Minimum width based on header
        let mut max_path_len = "Path".len(); // Minimum width based on header

        // Calculate max lengths for metadata columns
        let mut max_metadata_lengths = Vec::new();
        for key in &sorted_metadata_keys {
            // Initialize with header length
            max_metadata_lengths.push(key.len());
        }

        // Iterate through assets to find max content lengths
        for asset in &app.assets {
            // Update max name length
            max_name_len = std::cmp::max(max_name_len, asset.name.len());

            // Update max path length
            max_path_len = std::cmp::max(max_path_len, asset.folder_uuid.len());

            // Update max metadata lengths
            if let Some(obj) = asset.metadata.as_object() {
                for (i, key) in sorted_metadata_keys.iter().enumerate() {
                    if let Some(value) = obj.get(key) {
                        // Handle string values to get actual length without quotes
                        let value_str = if let Some(str_val) = value.as_str() {
                            str_val
                        } else {
                            &value.to_string()
                        };

                        if i < max_metadata_lengths.len() {
                            max_metadata_lengths[i] = std::cmp::max(max_metadata_lengths[i], value_str.len());
                        }
                    }
                }
            }
        }

        // Create constraints based on calculated widths - optimizing for minimal real estate
        let mut widths = vec![
            Constraint::Length((max_icon_len + 1) as u16),  // Icon column with minimal padding
            Constraint::Length((max_name_len + 1) as u16), // Name column with minimal padding
            Constraint::Length((max_path_len + 1) as u16), // Path column with minimal padding
        ];

        // Add constraints for each metadata column with minimal padding
        for max_len in max_metadata_lengths {
            widths.push(Constraint::Length((max_len + 1) as u16)); // Minimal padding of 1 character
        }

        widths
    };

    if app.assets_loading_for_selection {
        // Show a loading indicator in a centered way with the frame
        let loading_text = Paragraph::new("⏳ Loading assets...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 149, 237))); // Cornflower blue

        f.render_widget(loading_text, area);
    } else if app.assets.is_empty() {
        // Show a centered "No data to display" message with the frame
        let no_data_text = Paragraph::new("No data to display")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 100, 100))); // Muted gray

        f.render_widget(no_data_text, area);
    } else {
        // Create table rows
        let rows = app.assets
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let is_selected = i == app.selected_asset_index;
                let row_style = if is_selected {
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

                // Create cells for the basic columns
                let mut cells = vec![
                    Cell::from(icon), // Icon cell
                    Cell::from(asset.name.as_str()), // Name cell
                    Cell::from(asset.folder_uuid.as_str()), // Path cell
                ];

                // Add cells for each metadata key
                if let Some(obj) = asset.metadata.as_object() {
                    for key in &sorted_metadata_keys {
                        // Special handling for the case where actual metadata is nested under a "meta" key
                        let value = if obj.contains_key("meta") {
                            // Check if the key exists in the nested "meta" object
                            if let Some(meta_obj) = obj.get("meta").and_then(|v| v.as_object()) {
                                if let Some(meta_value) = meta_obj.get(key) {
                                    // Handle string values to remove quotes
                                    if let Some(str_val) = meta_value.as_str() {
                                        str_val.to_string()
                                    } else {
                                        meta_value.to_string() // For non-string values, keep the JSON representation
                                    }
                                } else {
                                    "".to_string()
                                }
                            } else {
                                // If "meta" doesn't contain an object, fall back to regular lookup
                                if let Some(value) = obj.get(key) {
                                    if let Some(str_val) = value.as_str() {
                                        str_val.to_string()
                                    } else {
                                        value.to_string() // For non-string values, keep the JSON representation
                                    }
                                } else {
                                    "".to_string()
                                }
                            }
                        } else {
                            // Regular lookup if there's no "meta" key
                            if let Some(value) = obj.get(key) {
                                if let Some(str_val) = value.as_str() {
                                    str_val.to_string()
                                } else {
                                    value.to_string() // For non-string values, keep the JSON representation
                                }
                            } else {
                                "".to_string()
                            }
                        };
                        cells.push(create_cell_with_alignment(value));
                    }
                } else {
                    // If no metadata, add empty cells for all metadata columns
                    for _ in &sorted_metadata_keys {
                        cells.push(create_cell_with_alignment("".to_string()));
                    }
                }

                Row::new(cells).style(row_style)
            })
            .collect::<Vec<Row>>();

        // Create the table
        let table = Table::new(
            rows,
            column_widths,
        )
            .header(
                Row::new(headers.iter().map(|&h| Cell::from(h)))
                .style(Style::default().fg(Color::Rgb(255, 215, 0))) // Gold header text
                .bottom_margin(1)
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            )
            .highlight_style(Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)) // Forest green highlight
            .column_spacing(1); // Add spacing between columns for better readability

        f.render_widget(table, area);
    }
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
        crate::app::AppState::Log => "↑↓:scroll | ←→:h-scroll | c:COPY | C:CMD-COPY | q:quit",
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
                    " 📝 Log2 [{}/{}] ", // Added log emoji
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
        " 📝 Log2 [{}/{}] ",
        app.log_scroll_position + 1,
        app.log_entries.len()
    );

    // Create list items - we'll handle selection through ListState
    let start_idx = app.log_scroll_position.saturating_sub(10); // Show 10 entries before current position
    let end_idx = std::cmp::min(start_idx + 20, app.log_entries.len()); // Show 20 entries total

    let list_items: Vec<ListItem> = app
        .log_entries
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|entry| {
            // Apply horizontal scrolling by slicing the entry text
            let display_text = if app.log_horizontal_scroll > 0 {
                let scroll_offset = app.log_horizontal_scroll as usize;
                if entry.len() > scroll_offset {
                    // Safely slice the string by bytes, but try to avoid cutting in the middle of a character
                    let chars: Vec<char> = entry.chars().collect();
                    if chars.len() > scroll_offset {
                        chars[scroll_offset..].iter().collect::<String>()
                    } else {
                        "".to_string()
                    }
                } else {
                    entry.clone()
                }
            } else {
                entry.clone()
            };

            // Create a simple list item with the display text
            ListItem::new(Line::from(display_text))
        })
        .collect();

    // Create ListState to manage selection
    let mut state = ratatui::widgets::ListState::default();
    // Calculate the relative index of the selected item within the visible range
    let relative_selected_index = if app.log_scroll_position >= start_idx && end_idx > start_idx {
        Some(app.log_scroll_position - start_idx)
    } else {
        None
    };
    state.select(relative_selected_index);

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(
                    Style::default()
                        .fg(Color::Rgb(100, 149, 237)) // Cornflower blue border
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .style(
            Style::default()
                .bg(Color::Rgb(30, 30, 30)) // Same background as other panes
                .fg(Color::Rgb(200, 200, 200)), // Same text color as other panes
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(70, 130, 180)) // Steel blue
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" >> ")
        .scroll_padding(5); // Show 5 items before and after the selected item

    f.render_stateful_widget(list, area, &mut state);
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

// Helper function to determine if a value is numeric and format it appropriately
fn create_cell_with_alignment(value: String) -> Cell<'static> {
    // Try to parse as a number (integer or float)
    if value.parse::<f64>().is_ok() {
        // If it's a valid number, right-align it by wrapping it in a right-aligned Line
        Cell::from(Line::from(Span::raw(value)).alignment(Alignment::Right))
    } else {
        // For non-numeric values, use default left alignment
        Cell::from(value)
    }
}

// Generic function to extract metadata keys from a list of assets
fn extract_metadata_keys(assets: &[(Asset, f64)]) -> Vec<String> {
    let mut all_metadata_keys = std::collections::HashSet::<String>::new();
    for (asset, _) in assets {
        if let Some(obj) = asset.metadata.as_object() {
            for key in obj.keys() {
                // Special handling for the case where metadata contains a "meta" key that wraps actual metadata
                if key == "meta" {
                    // If the "meta" key contains an object, extract keys from that object instead
                    if let Some(meta_obj) = obj.get(key).and_then(|v| v.as_object()) {
                        for meta_key in meta_obj.keys() {
                            all_metadata_keys.insert(meta_key.clone());
                        }
                    } else {
                        // If "meta" doesn't contain an object, use it as a regular key
                        all_metadata_keys.insert(key.clone());
                    }
                } else {
                    // Regular handling for other keys
                    all_metadata_keys.insert(key.clone());
                }
            }
        }
    }

    // Convert to sorted vector
    let mut sorted_metadata_keys: Vec<String> = all_metadata_keys.into_iter().collect();
    sorted_metadata_keys.sort();
    sorted_metadata_keys
}

fn draw_asset_details_modal(f: &mut Frame, area: Rect, app: &App) {
    // Create a medium-sized centered modal window (60% width, 80% height)
    let popup_area = centered_rect(60, 80, area);

    // Clear the background first
    f.render_widget(Clear, popup_area);

    // Draw outer frame for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))  // Gold border
        .title(" 📋 Asset Details ")
        .style(Style::default().bg(Color::Rgb(30, 30, 40))); // Dark background matching theme

    f.render_widget(modal_block, popup_area);

    // Calculate inner area for content
    let inner_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    // Check if asset details are available
    if let Some(details) = &app.selected_asset_details {
        // Create a layout for the content area
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top padding
                Constraint::Min(1),    // Content area
                Constraint::Length(3), // Bottom space for close instruction
            ])
            .split(inner_area);

        // Create a paragraph widget to display asset details
        let details_text = vec![
            Line::from(vec![
                Span::styled("UUID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.uuid),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.name),
            ]),
            Line::from(vec![
                Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.path),
            ]),
            Line::from(vec![
                Span::styled("File Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.file_type),
            ]),
            Line::from(vec![
                Span::styled("File Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(match details.file_size {
                    Some(size) => format!("{}", size),
                    None => "Unknown".to_string(),
                }),
            ]),
            Line::from(vec![
                Span::styled("Processing Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.processing_status),
            ]),
            Line::from(vec![
                Span::styled("Created At: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.created_at),
            ]),
            Line::from(vec![
                Span::styled("Updated At: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.updated_at),
            ]),
            Line::from(vec![
                Span::styled("Is Assembly: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if details.is_assembly { "Yes" } else { "No" }),
            ]),
            Line::from(vec![
                Span::styled("Tenant ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.tenant_id),
            ]),
            Line::from(vec![
                Span::styled("Folder ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.folder_id),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&details.state),
            ]),
            Line::from(Span::raw("")), // Empty line before metadata
            Line::from(vec![
                Span::styled("Metadata:", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            ]),
        ];

        // Add metadata fields if they exist
        let mut metadata_lines = Vec::new();
        if let Some(obj) = details.metadata.as_object() {
            for (key, value) in obj {
                let value_str = if let Some(str_val) = value.as_str() {
                    str_val.to_string()
                } else {
                    value.to_string() // For non-string values, keep the JSON representation
                };

                metadata_lines.push(Line::from(vec![
                    Span::styled(format!("  {}: ", key), Style::default().add_modifier(Modifier::ITALIC)),
                    Span::raw(value_str),
                ]));
            }
        } else if !details.metadata.is_null() {
            // If metadata is not an object but exists, show it as a single line
            metadata_lines.push(Line::from(vec![
                Span::styled("  Metadata: ", Style::default().add_modifier(Modifier::ITALIC)),
                Span::raw(details.metadata.to_string()),
            ]));
        }

        // Combine all lines
        let mut all_lines = details_text;
        all_lines.extend(metadata_lines);

        let details_paragraph = Paragraph::new(all_lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((0, 0))
            .style(Style::default().fg(Color::Rgb(200, 200, 200)));

        // Render the details paragraph
        f.render_widget(details_paragraph, content_chunks[1]);

        // Add close instruction at the bottom
        let close_instruction = Paragraph::new("Press 'q' or ESC to close")
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM));

        f.render_widget(close_instruction, content_chunks[2]);
    } else {
        // Show a message if no asset details are available
        let no_details_text = Paragraph::new("No asset details available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 100, 100)));

        f.render_widget(no_details_text, inner_area);
    }
}

fn draw_geometric_match_modal(f: &mut Frame, area: Rect, app: &App) {
    // Create a larger centered modal window (80% of screen)
    let popup_area = centered_rect(80, 80, area);

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

    // Extract metadata keys using the generic function
    let sorted_metadata_keys = extract_metadata_keys(&app.geometric_match_results);

    // Calculate width for each column based on max content length
    let column_widths = if app.geometric_match_results.is_empty() {
        // Default widths when no results
        let mut widths = vec![
            Constraint::Length(3),  // Icon column
            Constraint::Min(15),    // Name column
            Constraint::Min(15),    // Folder Path column
            Constraint::Length(12), // Similarity Score column
        ];

        for _ in &sorted_metadata_keys {
            widths.push(Constraint::Min(10));
        }
        widths
    } else {
        // Calculate max lengths for each column
        let max_icon_len = 1; // Icons are single characters (don't need mut)
        let mut max_name_len = "Name".len(); // Minimum width based on header
        let mut max_path_len = "Folder Path".len(); // Minimum width based on header
        let mut max_similarity_len = "Similarity".len(); // Minimum width based on header

        // Calculate max lengths for metadata columns
        let mut max_metadata_lengths = Vec::new();
        for key in &sorted_metadata_keys {
            // Initialize with header length
            max_metadata_lengths.push(key.len());
        }

        // Iterate through results to find max content lengths
        for (asset, similarity_score) in &app.geometric_match_results {
            // Update max name length
            max_name_len = std::cmp::max(max_name_len, asset.name.len());

            // Update max path length
            let folder_path = asset.path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(&asset.path);
            max_path_len = std::cmp::max(max_path_len, folder_path.len());

            // Update max similarity length
            let similarity_text = format!("{:.2}%", (similarity_score * 100.0).round() / 100.0);
            max_similarity_len = std::cmp::max(max_similarity_len, similarity_text.len());

            // Update max metadata lengths
            if let Some(obj) = asset.metadata.as_object() {
                for (i, key) in sorted_metadata_keys.iter().enumerate() {
                    if let Some(value) = obj.get(key) {
                        // Handle string values to get actual length without quotes
                        let value_str = if let Some(str_val) = value.as_str() {
                            str_val
                        } else {
                            &value.to_string()
                        };

                        if i < max_metadata_lengths.len() {
                            max_metadata_lengths[i] = std::cmp::max(max_metadata_lengths[i], value_str.len());
                        }
                    }
                }
            }
        }

        // Create constraints based on calculated widths - optimizing for minimal real estate
        let mut widths = vec![
            Constraint::Length(max_icon_len as u16 + 1),  // Icon column with minimal padding
            Constraint::Length(max_name_len as u16 + 1), // Name column with minimal padding
            Constraint::Length(max_path_len as u16 + 1), // Folder Path column with minimal padding
            Constraint::Length(max_similarity_len as u16 + 1), // Similarity Score column with minimal padding
        ];

        // Add constraints for each metadata column with minimal padding
        for max_len in max_metadata_lengths {
            widths.push(Constraint::Length((max_len + 1) as u16)); // Minimal padding of 1 character
        }
        widths
    };

    if app.command_in_progress {
        // Show a searching indicator when command is in progress with the frame
        let searching_text = Paragraph::new("⏳ Processing geometric match...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)) // Gold border
                    .title(" 🔍 Geometric Match Results "), // Title for consistency
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));

        f.render_widget(searching_text, inner_area);
    } else if app.geometric_match_results.is_empty() {
        // Show a centered "No data to display" message with the frame
        let no_data_text = Paragraph::new("No data to display")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)) // Gold border
                    .title(format!(" Results ({}) ", app.geometric_match_results.len())), // Title with count
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 100, 100))); // Muted gray

        f.render_widget(no_data_text, inner_area);
    } else {
        // Create table rows
        let rows = app.geometric_match_results
            .iter()
            .enumerate()
            .map(|(i, (asset, similarity_score))| {
                let is_selected = i == app.geometric_match_scroll_position; // Use geometric match scroll position
                let row_style = if is_selected {
                    Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White) // Forest green to match other selections
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200)) // Light gray for readability
                };

                let icon = match asset.file_type.as_str() {
                    "model" => "🏗️",    // Building/construction icon for 3D models
                    "document" => "📝", // Document icon
                    "image" => "🖼️",    // Image icon
                    "video" => "🎥",    // Video icon
                    "audio" => "🎧",    // Audio icon
                    "archive" => "📦",  // Archive icon
                    "code" => "💻",     // Code/icon
                    _ => "📁",          // Default folder icon
                };

                // Format the similarity score as a percentage with right alignment
                let similarity_percent = (similarity_score * 100.0).round() / 100.0; // Round to 2 decimal places
                let similarity_formatted = format!("{:>8.2}%", similarity_percent); // Right-align with padding
                let similarity_cell = Cell::from(similarity_formatted)
                    .style(if is_selected {
                        Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::Rgb(173, 216, 230)) // Lighter text for similarity in selected item
                    } else {
                        Style::default().fg(Color::Rgb(173, 216, 230)) // Light blue for similarity in unselected items
                    });

                // Extract folder path from asset path
                let folder_path = asset.path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(&asset.path);

                // Create cells for the basic columns
                let mut cells = vec![
                    Cell::from(icon), // Icon cell
                    Cell::from(asset.name.as_str()), // Name cell (left-aligned by default)
                    Cell::from(folder_path), // Folder Path cell (left-aligned by default)
                    similarity_cell, // Similarity cell (right-aligned)
                ];

                // Add cells for each metadata key
                if let Some(obj) = asset.metadata.as_object() {
                    for key in &sorted_metadata_keys {
                        // Special handling for the case where actual metadata is nested under a "meta" key
                        let value = if obj.contains_key("meta") {
                            // Check if the key exists in the nested "meta" object
                            if let Some(meta_obj) = obj.get("meta").and_then(|v| v.as_object()) {
                                if let Some(meta_value) = meta_obj.get(key) {
                                    // Handle string values to remove quotes
                                    if let Some(str_val) = meta_value.as_str() {
                                        str_val.to_string()
                                    } else {
                                        meta_value.to_string() // For non-string values, keep the JSON representation
                                    }
                                } else {
                                    "".to_string()
                                }
                            } else {
                                // If "meta" doesn't contain an object, fall back to regular lookup
                                if let Some(value) = obj.get(key) {
                                    if let Some(str_val) = value.as_str() {
                                        str_val.to_string()
                                    } else {
                                        value.to_string() // For non-string values, keep the JSON representation
                                    }
                                } else {
                                    "".to_string()
                                }
                            }
                        } else {
                            // Regular lookup if there's no "meta" key
                            if let Some(value) = obj.get(key) {
                                if let Some(str_val) = value.as_str() {
                                    str_val.to_string()
                                } else {
                                    value.to_string() // For non-string values, keep the JSON representation
                                }
                            } else {
                                "".to_string()
                            }
                        };
                        cells.push(create_cell_with_alignment(value));
                    }
                } else {
                    // If metadata is not an object, add empty cells for all metadata columns
                    for _ in &sorted_metadata_keys {
                        cells.push(create_cell_with_alignment("".to_string()));
                    }
                }

                Row::new(cells).style(row_style)
            })
            .collect::<Vec<Row>>();

        // Create headers for the table
        let mut headers = vec![
            Cell::from(""), // Empty header for icon column
            Cell::from("Name"),
            Cell::from("Folder Path"),
            Cell::from("Similarity  "), // Extra spaces to align with right-aligned content
        ];

        // Add headers for each metadata key
        for key in &sorted_metadata_keys {
            headers.push(Cell::from(key.as_str()));
        }

        // Create the table
        let table = Table::new(
            rows,
            column_widths,
        )
            .header(
                Row::new(headers)
                .style(Style::default().fg(Color::Rgb(255, 215, 0))) // Gold header text
                .bottom_margin(1)
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)) // Gold border
                    .title(format!(" Results ({}) ", app.geometric_match_results.len())), // Title with count
            )
            .highlight_style(Style::default().bg(Color::Rgb(34, 139, 34)).fg(Color::White)) // Forest green highlight
            .column_spacing(1); // Add spacing between columns for better readability

        // Render the table
        f.render_widget(table, inner_area);
    }
}
