use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};

use crate::pcli_commands;
use chrono::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    pub uuid: String,
    pub name: String,
    pub path: String,
    pub folders_count: u32,
    pub assets_count: u32,
    pub parent_uuid: Option<String>,
    pub children: Vec<Folder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub uuid: String,
    pub name: String,
    pub folder_uuid: String,
    pub file_type: String,
    pub size: Option<u64>,
    pub path: String,        // Add path field to store the full path
    pub metadata: serde_json::Value,  // Add metadata field
}

#[derive(Debug, Clone)]
pub struct FolderCache {
    pub folders: Vec<Folder>,
    pub assets: Vec<Asset>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Folders,
    Assets,
    Search,
    Uploading,
    Downloading,
    Help,
    CommandHistory,
    Log,
    PaneResize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePane {
    Folders,
    Assets,
    Log,
}

pub struct App {
    pub current_state: AppState,
    pub folders: Vec<Folder>,
    pub assets: Vec<Asset>,
    pub current_folder: Option<String>,
    pub selected_folder_index: usize,
    pub selected_asset_index: usize,
    pub search_query: String,
    pub status_message: String,
    pub should_quit: bool,
    pub active_pane: ActivePane,
    pub folder_cache: HashMap<String, FolderCache>,
    pub assets_loading_for_selection: bool, // Flag to indicate if assets are being loaded for selected folder
    pub last_executed_command: String,      // Track the last executed PCLI2 command
    pub command_history: Vec<String>,       // Track command history
    pub log_entries: Vec<String>,           // Track log entries (commands and outputs)
    pub log_scroll_position: usize,         // Track scroll position in log
    pub log_horizontal_scroll: u16,         // Track horizontal scroll position in log
    pub show_search_modal: bool,            // Whether to show the search modal
    pub search_input_buffer: String,        // Buffer for search input
    pub command_in_progress: bool,          // Whether a PCLI2 command is currently running
    pub resize_mode_active: bool,           // Whether pane resize mode is active
    pub resize_delta_x: i32,                // Horizontal resize adjustment
    pub resize_delta_y: i32,                // Vertical resize adjustment
    pub search_results: Vec<Asset>,          // Store search results separately from folder assets
    pub search_modal_focus: SearchModalFocus, // Track which element has focus in search modal
    pub selected_search_result_index: usize,  // Track selected index in search results separately
    pub geometric_match_results: Vec<(Asset, f64)>,  // Store geometric match results with similarity scores
    pub show_geometric_match_modal: bool,     // Whether to show the geometric match modal
    pub geometric_match_scroll_position: usize, // Track scroll position in geometric match results
    pub geometric_match_horizontal_scroll: u16, // Track horizontal scroll position for many columns
    pub show_asset_details_modal: bool,       // Whether to show the asset details modal
    pub selected_asset_details: Option<AssetDetails>, // Details of the selected asset
    pub last_entered_folder_path: Option<String>, // Track the last folder entered to re-select it when going back
    pub clipboard: Option<arboard::Clipboard>, // Clipboard for copying log entries
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("current_state", &self.current_state)
            .field("folders", &self.folders)
            .field("assets", &self.assets)
            .field("current_folder", &self.current_folder)
            .field("selected_folder_index", &self.selected_folder_index)
            .field("selected_asset_index", &self.selected_asset_index)
            .field("search_query", &self.search_query)
            .field("status_message", &self.status_message)
            .field("should_quit", &self.should_quit)
            .field("active_pane", &self.active_pane)
            .field("folder_cache", &self.folder_cache)
            .field("assets_loading_for_selection", &self.assets_loading_for_selection)
            .field("last_executed_command", &self.last_executed_command)
            .field("command_history", &self.command_history)
            .field("log_entries", &self.log_entries)
            .field("log_scroll_position", &self.log_scroll_position)
            .field("show_search_modal", &self.show_search_modal)
            .field("search_input_buffer", &self.search_input_buffer)
            .field("command_in_progress", &self.command_in_progress)
            .field("resize_mode_active", &self.resize_mode_active)
            .field("resize_delta_x", &self.resize_delta_x)
            .field("resize_delta_y", &self.resize_delta_y)
            .field("search_results", &self.search_results)
            .field("search_modal_focus", &self.search_modal_focus)
            .field("selected_search_result_index", &self.selected_search_result_index)
            .field("geometric_match_results", &self.geometric_match_results)
            .field("show_geometric_match_modal", &self.show_geometric_match_modal)
            .field("show_asset_details_modal", &self.show_asset_details_modal)
            .field("selected_asset_details", &self.selected_asset_details)
            .field("last_entered_folder_path", &self.last_entered_folder_path)
            .field("clipboard", &"Option<Clipboard>") // Skip printing clipboard content
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchModalFocus {
    Input,
    Results,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AssetDetails {
    pub uuid: String,
    pub name: String,
    pub path: String,
    pub file_type: String,
    pub file_size: Option<u64>,
    pub processing_status: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_assembly: bool,
    pub tenant_id: String,
    pub folder_id: String,
    pub state: String,
    pub metadata: serde_json::Value,  // Add metadata field
}

impl App {
    pub fn new() -> Self {
        Self {
            current_state: AppState::Folders,
            folders: vec![],
            assets: vec![],
            current_folder: None,
            selected_folder_index: 0,
            selected_asset_index: 0,
            search_query: String::new(),
            status_message: "Ready".to_string(),
            should_quit: false,
            active_pane: ActivePane::Folders,
            folder_cache: HashMap::new(),
            assets_loading_for_selection: false,
            last_executed_command: String::new(),
            command_history: Vec::new(),
            log_entries: Vec::new(),
            log_scroll_position: 0,
            log_horizontal_scroll: 0,
            show_search_modal: false,
            search_input_buffer: String::new(),
            command_in_progress: false,
            resize_mode_active: false,
            resize_delta_x: 0,
            resize_delta_y: 0,
            search_results: vec![],
            search_modal_focus: SearchModalFocus::Input,
            selected_search_result_index: 0,
            geometric_match_results: vec![],
            show_geometric_match_modal: false,
            geometric_match_scroll_position: 0,
            geometric_match_horizontal_scroll: 0,
            show_asset_details_modal: false,
            selected_asset_details: None,
            last_entered_folder_path: None,
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub fn copy_selected_log_entry_to_clipboard(&mut self) {
        if !self.log_entries.is_empty() && self.log_scroll_position < self.log_entries.len() {
            let log_entry = &self.log_entries[self.log_scroll_position];

            if let Some(ref mut clipboard) = self.clipboard {
                if let Err(e) = clipboard.set_text(log_entry) {
                    self.status_message = format!("Failed to copy to clipboard: {}", e);
                } else {
                    self.status_message = "Log entry copied to clipboard".to_string();
                }
            } else {
                self.status_message = "Clipboard not available".to_string();
            }
        }
    }

    pub fn copy_command_from_selected_log_entry_to_clipboard(&mut self) {
        if !self.log_entries.is_empty() && self.log_scroll_position < self.log_entries.len() {
            let log_entry = &self.log_entries[self.log_scroll_position];

            // Extract just the command part from the log entry
            // Log entries typically follow the format: "[HH:MM:SS] STATUS: command"
            let command_part = self.extract_command_from_log_entry(log_entry);

            if let Some(ref mut clipboard) = self.clipboard {
                if let Err(e) = clipboard.set_text(&command_part) {
                    self.status_message = format!("Failed to copy command to clipboard: {}", e);
                } else {
                    self.status_message = "Command copied to clipboard".to_string();
                }
            } else {
                self.status_message = "Clipboard not available".to_string();
            }
        }
    }

    fn extract_command_from_log_entry(&self, log_entry: &str) -> String {
        // Look for patterns like "[HH:MM:SS] STATUS: command" or "[HH:MM:SS] STATUS: command"
        // Common patterns in our logs:
        // - "[HH:MM:SS] ✓ SUCCESS: command"
        // - "[HH:MM:SS] ✗ ERROR: command - error details"
        // - "[HH:MM:SS] CACHED: command"

        // Find the colon after the timestamp and status
        if let Some(colon_pos) = log_entry.find(": ") {
            // Skip past the timestamp and status part
            let after_colon = &log_entry[colon_pos + 2..];

            // If there's an error part after a " - ", only return the command part
            if let Some(error_separator) = after_colon.find(" - ") {
                return after_colon[..error_separator].trim().to_string();
            } else {
                return after_colon.trim().to_string();
            }
        }

        // If we can't parse it, return the whole entry
        log_entry.to_string()
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle geometric match modal if it's active - make it modal and prevent other interactions
        if self.show_geometric_match_modal {
            self.handle_geometric_match_keys(key).await;
            return;
        }

        // Handle asset details modal if it's active
        if self.show_asset_details_modal {
            // Handle closing the asset details modal
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                self.show_asset_details_modal = false;
                return;
            }
        }

        // Handle search modal if it's active - make it modal and prevent other interactions
        if self.show_search_modal {
            self.handle_search_keys(key).await;
            return;
        }

        // Handle global keys that work in any state
        // Only allow pane cycling when search modal is not active
        if key.code == KeyCode::Tab && !key.modifiers.contains(crossterm::event::KeyModifiers::ALT)
        {
            // Cycle between panes forward (Tab without Alt)
            self.active_pane = match self.active_pane {
                ActivePane::Folders => ActivePane::Assets,
                ActivePane::Assets => ActivePane::Log,
                ActivePane::Log => ActivePane::Folders,
            };
            return;
        } else if key.code == KeyCode::BackTab
            || (key.code == KeyCode::Tab
                && key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT))
        {
            // Cycle between panes in reverse order (Shift+Tab or BackTab)
            self.active_pane = match self.active_pane {
                ActivePane::Folders => ActivePane::Log,
                ActivePane::Assets => ActivePane::Folders,
                ActivePane::Log => ActivePane::Assets,
            };
            return;
        }

        // Handle pane resize mode activation (Ctrl+N)
        if key.code == KeyCode::Char('n')
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.resize_mode_active = true;
            self.current_state = AppState::PaneResize;
            self.status_message =
                "Resize mode: Use arrow keys to resize, Enter to confirm, Esc to cancel"
                    .to_string();
            return;
        }

        // Handle help key globally
        if key.code == KeyCode::Char('h') {
            self.current_state = AppState::Help;
            return;
        }

        // Handle command history key globally
        if key.code == KeyCode::Char('c') {
            self.current_state = AppState::CommandHistory;
            return;
        }

        // Handle search key globally - show modal instead of changing state
        if key.code == KeyCode::Char('/') {
            self.show_search_modal = true;
            self.search_input_buffer.clear();
            return;
        }

        // Handle log view key globally, but only when search modal is not active
        if key.code == KeyCode::Char('l') && !self.show_search_modal && !self.show_geometric_match_modal {
            self.current_state = AppState::Log;
            return;
        }

        // Handle search modal if it's active
        if self.show_search_modal {
            self.handle_search_keys(key).await;
            return;
        }

        match self.current_state {
            AppState::Folders => self.handle_folder_keys(key).await,
            AppState::Assets => self.handle_asset_keys(key).await,
            AppState::Search => self.handle_search_keys(key).await,
            AppState::Uploading => {
                // Handle upload specific keys
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.current_state = AppState::Folders;
                        self.status_message = "Upload mode exited".to_string();
                    }
                    KeyCode::Char('u') => {
                        // Trigger interactive upload
                        self.upload_asset_interactive().await;
                    }
                    _ => {}
                }
            }
            AppState::Downloading => {
                // Handle download specific keys
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    self.current_state = AppState::Folders;
                    self.status_message = "Download mode exited".to_string();
                }
            }
            AppState::Help => {
                // Handle help specific keys
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    // Return to the previous state (default to Folders)
                    self.current_state = AppState::Folders;
                }
            }
            AppState::CommandHistory => {
                // Handle command history specific keys
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Return to the previous state (default to Folders)
                        self.current_state = AppState::Folders;
                    }
                    _ => {}
                }
            }
            AppState::Log => {
                // Handle log specific keys
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Return to the previous state (default to Folders)
                        self.current_state = AppState::Folders;
                    }
                    KeyCode::Up => {
                        // Scroll up in the log
                        if self.log_scroll_position > 0 {
                            self.log_scroll_position -= 1;
                        }
                    }
                    KeyCode::Down => {
                        // Scroll down in the log
                        if self.log_scroll_position < self.log_entries.len().saturating_sub(1) {
                            self.log_scroll_position += 1;
                        }
                    }
                    KeyCode::Left => {
                        // Scroll left in the log horizontally
                        if self.log_horizontal_scroll > 0 {
                            self.log_horizontal_scroll -= 1;
                        }
                    }
                    KeyCode::Right => {
                        // Scroll right in the log horizontally
                        // Increase the horizontal scroll position
                        self.log_horizontal_scroll += 1;
                    }
                    KeyCode::Char('c') => {
                        // Copy selected log entry to clipboard
                        self.copy_selected_log_entry_to_clipboard();
                    }
                    KeyCode::Char('C') => {
                        // Copy just the command part from the selected log entry to clipboard
                        self.copy_command_from_selected_log_entry_to_clipboard();
                    }
                    _ => {}
                }
            }
            AppState::PaneResize => self.handle_resize_keys(key).await,
        }
    }

    async fn handle_folder_keys(&mut self, key: KeyEvent) {
        let prev_selected_folder_index = self.selected_folder_index;

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => match self.active_pane {
                ActivePane::Folders => {
                    if !self.folders.is_empty() {
                        self.selected_folder_index =
                            (self.selected_folder_index + 1).min(self.folders.len() - 1);
                    }
                }
                ActivePane::Assets => {
                    if !self.assets.is_empty() {
                        self.selected_asset_index =
                            (self.selected_asset_index + 1).min(self.assets.len() - 1);
                    }
                }
                ActivePane::Log => {
                    // Scroll down in the log
                    if self.log_scroll_position < self.log_entries.len().saturating_sub(1) {
                        self.log_scroll_position += 1;
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.active_pane {
                ActivePane::Folders => {
                    if self.selected_folder_index > 0 {
                        self.selected_folder_index -= 1;
                    }
                }
                ActivePane::Assets => {
                    if self.selected_asset_index > 0 {
                        self.selected_asset_index -= 1;
                    }
                }
                ActivePane::Log => {
                    // Scroll up in the log
                    if self.log_scroll_position > 0 {
                        self.log_scroll_position -= 1;
                    }
                }
            },
            KeyCode::Enter => {
                match self.active_pane {
                    ActivePane::Folders => {
                        if !self.folders.is_empty()
                            && self.selected_folder_index < self.folders.len()
                        {
                            let folder = &self.folders[self.selected_folder_index];

                            // Check if this is the parent directory indicator
                            if folder.uuid == ".." {
                                self.go_back_to_parent_folder().await;

                                // After going back to parent, load assets for the parent folder
                                self.load_assets_for_current_folder().await;
                            } else {
                                self.enter_folder(folder.path.clone()).await; // Use the full path
                            }
                        }
                    }
                    ActivePane::Assets => {
                        // Show detailed information for the selected asset
                        if !self.assets.is_empty() && self.selected_asset_index < self.assets.len() {
                            self.show_asset_details();
                        }
                    }
                    ActivePane::Log => {
                        // Perform action on selected log entry if needed
                        // For now, just do nothing
                    }
                }

                // After entering a folder, we should return to avoid loading assets for selection
                return;
            }
            KeyCode::Char('a') => {
                self.switch_to_assets_view().await;
            }
            KeyCode::Char('/') => {
                self.current_state = AppState::Search;
            }
            KeyCode::Char('u') => {
                self.current_state = AppState::Uploading;
                self.status_message = "Upload mode activated. Press 'q' to return.".to_string();
            }
            KeyCode::Char('d') => {
                self.current_state = AppState::Downloading;
                self.status_message = "Download mode activated. Press 'q' to return.".to_string();
            }
            KeyCode::Char('g') => {
                // Perform geometric match on selected asset when in Folders state but Assets pane is active
                if self.active_pane == ActivePane::Assets && !self.assets.is_empty() && self.selected_asset_index < self.assets.len() {
                    let asset_uuid = self.assets[self.selected_asset_index].uuid.clone();
                    let asset_name = self.assets[self.selected_asset_index].name.clone();

                    self.perform_geometric_match(&asset_uuid).await;
                    self.show_geometric_match_modal = true; // Show the geometric match modal
                    self.status_message = format!("Geometric match performed on: {}", asset_name);
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back_to_parent_folder().await;
            }
            _ => {}
        }

        // If the selected folder index changed in the folders pane, load assets for the selected folder
        if self.active_pane == ActivePane::Folders
            && prev_selected_folder_index != self.selected_folder_index
        {
            self.load_assets_for_selected_folder().await;
        }
    }

    async fn handle_asset_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => match self.active_pane {
                ActivePane::Assets => {
                    if !self.assets.is_empty() {
                        self.selected_asset_index =
                            (self.selected_asset_index + 1).min(self.assets.len() - 1);
                    }
                }
                ActivePane::Folders => {
                    if !self.folders.is_empty() {
                        self.selected_folder_index =
                            (self.selected_folder_index + 1).min(self.folders.len() - 1);
                    }
                }
                ActivePane::Log => {
                    // Scroll down in the log
                    if self.log_scroll_position < self.log_entries.len().saturating_sub(1) {
                        self.log_scroll_position += 1;
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.active_pane {
                ActivePane::Assets => {
                    if self.selected_asset_index > 0 {
                        self.selected_asset_index -= 1;
                    }
                }
                ActivePane::Folders => {
                    if self.selected_folder_index > 0 {
                        self.selected_folder_index -= 1;
                    }
                }
                ActivePane::Log => {
                    // Scroll up in the log
                    if self.log_scroll_position > 0 {
                        self.log_scroll_position -= 1;
                    }
                }
            },
            KeyCode::Enter => {
                match self.active_pane {
                    ActivePane::Assets => {
                        // Show detailed information for the selected asset
                        if !self.assets.is_empty() && self.selected_asset_index < self.assets.len() {
                            self.show_asset_details();
                        }
                    }
                    ActivePane::Folders => {
                        if !self.folders.is_empty()
                            && self.selected_folder_index < self.folders.len()
                        {
                            let folder = &self.folders[self.selected_folder_index];

                            // Check if this is the parent directory indicator
                            if folder.uuid == ".." {
                                self.go_back_to_parent_folder().await;

                                // After going back to parent, load assets for the parent folder
                                self.load_assets_for_current_folder().await;
                            } else {
                                self.enter_folder(folder.path.clone()).await; // Use the full path
                            }
                        }
                    }
                    ActivePane::Log => {
                        // Perform action on selected log entry if needed
                        // For now, just do nothing
                    }
                }
            },
            KeyCode::Char('g') => {
                // Perform geometric match on selected asset
                if !self.assets.is_empty() && self.selected_asset_index < self.assets.len() {
                    let asset_uuid = self.assets[self.selected_asset_index].uuid.clone();
                    let asset_name = self.assets[self.selected_asset_index].name.clone();

                    self.perform_geometric_match(&asset_uuid).await;
                    self.show_geometric_match_modal = true; // Show the geometric match modal
                    self.status_message = format!("Geometric match performed on: {}", asset_name);
                }
            },
            KeyCode::Char('d') => {
                // Download selected asset
                if !self.assets.is_empty() && self.selected_asset_index < self.assets.len() {
                    let asset_uuid = self.assets[self.selected_asset_index].uuid.clone();
                    let asset_name = self.assets[self.selected_asset_index].name.clone();
                    self.download_asset_by_uuid(&asset_uuid, &asset_name).await;
                }
            }
            KeyCode::Char('q') => {
                // Go back to folder view
                self.current_state = AppState::Folders;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back_to_parent_folder().await;
            }
            _ => {}
        }
    }

    async fn handle_search_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c != '\n' => {
                // Only add character if we're focused on the input field
                if matches!(self.search_modal_focus, SearchModalFocus::Input) {
                    self.search_input_buffer.push(c);
                }
            }
            KeyCode::Tab => {
                // Cyclic focus forward: Input -> Results -> Input -> ...
                self.search_modal_focus = match self.search_modal_focus {
                    SearchModalFocus::Input => SearchModalFocus::Results,
                    SearchModalFocus::Results => SearchModalFocus::Input,
                };
            }
            KeyCode::BackTab => {
                // Cyclic focus backward: Results -> Input -> Results -> ...
                self.search_modal_focus = match self.search_modal_focus {
                    SearchModalFocus::Results => SearchModalFocus::Input,
                    SearchModalFocus::Input => SearchModalFocus::Results,
                };
            }
            KeyCode::Backspace => {
                // Only process backspace if focused on input
                if matches!(self.search_modal_focus, SearchModalFocus::Input) {
                    self.search_input_buffer.pop();
                }
            }
            KeyCode::Enter => {
                match self.search_modal_focus {
                    SearchModalFocus::Input => {
                        // Perform search when Enter is pressed in input field
                        self.search_query = self.search_input_buffer.clone();
                        self.perform_search().await;
                        // Switch focus to results after search
                        self.search_modal_focus = SearchModalFocus::Results;
                    }
                    SearchModalFocus::Results => {
                        // Allow user to go back to input field when Enter is pressed in results
                        self.search_modal_focus = SearchModalFocus::Input;
                    }
                }
            }
            KeyCode::Esc => {
                self.show_search_modal = false;
                self.search_input_buffer.clear();
                self.search_modal_focus = SearchModalFocus::Input; // Reset focus
            }
            KeyCode::Down => {
                // Navigate down in search results only if focused on results
                if matches!(self.search_modal_focus, SearchModalFocus::Results)
                    && !self.search_results.is_empty() {
                    self.selected_search_result_index =
                        (self.selected_search_result_index + 1).min(self.search_results.len() - 1);
                }
            }
            KeyCode::Up => {
                // Navigate up in search results only if focused on results
                if matches!(self.search_modal_focus, SearchModalFocus::Results)
                    && self.selected_search_result_index > 0 {
                    self.selected_search_result_index -= 1;
                }
            }
            KeyCode::Char('d')
                if matches!(self.search_modal_focus, SearchModalFocus::Results) &&
                   !self.search_results.is_empty() && self.selected_search_result_index < self.search_results.len() =>
            {
                // Download selected asset from search results
                let asset_uuid = self.search_results[self.selected_search_result_index].uuid.clone();
                let asset_name = self.search_results[self.selected_search_result_index].name.clone();
                self.download_asset_by_uuid(&asset_uuid, &asset_name).await;
            }
            _ => {}
        }
    }

    async fn handle_resize_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                // Increase vertical size of the current pane (decrease the one below)
                self.resize_delta_y -= 1;
                self.status_message =
                    format!("Resize: Adjusting vertically ({})", self.resize_delta_y);
            }
            KeyCode::Down => {
                // Decrease vertical size of the current pane (increase the one below)
                self.resize_delta_y += 1;
                self.status_message =
                    format!("Resize: Adjusting vertically ({})", self.resize_delta_y);
            }
            KeyCode::Left => {
                // Decrease horizontal size of the current pane (increase the one to the right)
                self.resize_delta_x -= 1;
                self.status_message =
                    format!("Resize: Adjusting horizontally ({})", self.resize_delta_x);
            }
            KeyCode::Right => {
                // Increase horizontal size of the current pane (decrease the one to the right)
                self.resize_delta_x += 1;
                self.status_message =
                    format!("Resize: Adjusting horizontally ({})", self.resize_delta_x);
            }
            KeyCode::Enter => {
                // Apply the resize changes and exit resize mode
                self.resize_mode_active = false;
                self.current_state = AppState::Folders; // Return to default state
                self.status_message = format!(
                    "Resize applied: dx={}, dy={}",
                    self.resize_delta_x, self.resize_delta_y
                );
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                // Cancel resize and return to previous state
                self.resize_mode_active = false;
                self.resize_delta_x = 0;
                self.resize_delta_y = 0;
                self.current_state = AppState::Folders; // Return to default state
                self.status_message = "Resize cancelled".to_string();
            }
            _ => {}
        }
    }

    pub async fn load_folders_for_current_context(&mut self) {
        match &self.current_folder {
            Some(current_path) => {
                // Check if we have cached data for this folder
                if let Some(cached_data) = self.folder_cache.get(current_path) {
                    // Check if cache is still valid (less than 5 minutes old)
                    if cached_data
                        .timestamp
                        .elapsed()
                        .unwrap_or(std::time::Duration::MAX)
                        < std::time::Duration::from_secs(300)
                    {
                        // 5 minutes
                        self.folders = cached_data.folders.clone();
                        self.assets = cached_data.assets.clone(); // Also update assets from cache
                        self.status_message =
                            format!("Loaded {} subfolders from cache", self.folders.len());
                        self.last_executed_command = format!(
                            "pcli2 folder list --folder-path \"{}\" --format json",
                            current_path
                        );
                        self.command_history
                            .push(self.last_executed_command.clone());
                        self.add_log_entry(format!("[{}] ✓ CACHED: {} (would have executed: pcli2 folder list --folder-path \"{}\" --format json)",
                            Local::now().format("%H:%M:%S"),
                            self.last_executed_command,
                            current_path));
                        return;
                    }
                }

                self.last_executed_command = format!(
                    "pcli2 folder list --folder-path \"{}\" --format json",
                    current_path
                );
                self.command_history
                    .push(self.last_executed_command.clone());
                self.command_in_progress = true; // Set flag when command starts
                self.status_message = format!("Loading subfolders for {}...", current_path);

                match pcli_commands::list_subfolders_of_folder(current_path) {
                    Ok(pcli_folders) => {
                        // Convert pcli folders to our internal representation
                        let mut folders: Vec<Folder> = pcli_folders
                            .into_iter()
                            .map(|f| Folder {
                                uuid: f.id, // Map 'id' from pcli to 'uuid' in our struct
                                name: f.name,
                                path: f.path, // Store the full path
                                folders_count: f.folders_count,
                                assets_count: f.assets_count,
                                parent_uuid: None, // pcli doesn't provide parent info in list
                                children: vec![],
                            })
                            .collect();

                        // Add parent directory indicator if we're not at the root
                        // Check if this is not a top-level folder (doesn't start with just the folder name)
                        if current_path.contains('/') {
                            if let Some(pos) = current_path.rfind('/') {
                                let parent_path = &current_path[..pos];
                                folders.insert(
                                    0,
                                    Folder {
                                        uuid: String::from(".."), // Special identifier for parent
                                        name: String::from(".."),
                                        path: parent_path.to_string(), // Parent path
                                        folders_count: 0,
                                        assets_count: 0,
                                        parent_uuid: None,
                                        children: vec![],
                                    },
                                );
                            }
                        } else if !current_path.is_empty() {
                            // If we're in a top-level folder, parent is root
                            folders.insert(
                                0,
                                Folder {
                                    uuid: String::from(".."), // Special identifier for parent
                                    name: String::from(".."),
                                    path: String::from(""), // Root path
                                    folders_count: 0,
                                    assets_count: 0,
                                    parent_uuid: None,
                                    children: vec![],
                                },
                            );
                        }

                        // Cache the folder data
                        let cache_entry = FolderCache {
                            folders: folders.clone(),
                            assets: self.assets.clone(), // Keep current assets in cache
                            timestamp: std::time::SystemTime::now(),
                        };
                        self.folder_cache.insert(current_path.clone(), cache_entry);

                        self.folders = folders;
                        self.status_message = format!("Loaded {} subfolders", self.folders.len());
                        self.command_in_progress = false; // Clear flag when command completes
                    }
                    Err(e) => {
                        self.status_message = format!("Error loading subfolders: {}", e);

                        // Log failed command with error indicator
                        self.add_log_entry(format!(
                            "[{}] ✗ ERROR: {} - {}",
                            Local::now().format("%H:%M:%S"),
                            self.last_executed_command,
                            e
                        ));
                        self.command_in_progress = false; // Clear flag when command completes
                    }
                }
            }
            None => {
                // If no specific folder is selected, load all top-level folders
                self.load_all_folders().await;
            }
        }
    }

    pub async fn load_assets_for_current_folder(&mut self) {
        if let Some(ref folder_path) = self.current_folder {
            self.last_executed_command = format!(
                "pcli2 asset list --folder-path \"{}\" --format json --metadata",
                folder_path
            );
            self.command_history
                .push(self.last_executed_command.clone());
            self.command_in_progress = true; // Set flag when command starts
            self.status_message = "Loading assets...".to_string();

            match pcli_commands::list_assets_in_folder(folder_path) {
                Ok(pcli_assets) => {
                    // Convert pcli assets to our internal representation
                    let assets: Vec<Asset> = pcli_assets
                        .into_iter()
                        .map(|a| Asset {
                            uuid: a.uuid,
                            name: a.name,
                            folder_uuid: self.current_folder.clone().unwrap_or_default(), // Use current folder as parent
                            file_type: a.file_type,
                            size: a.file_size,
                            path: a.path,
                            metadata: a.metadata,
                        })
                        .collect();

                    // Update or create cache entry with new asset data
                    // Always update the cache to ensure we have the latest data
                    let cache_entry = FolderCache {
                        folders: self.folders.clone(), // Keep current folders in cache
                        assets: assets.clone(),
                        timestamp: std::time::SystemTime::now(),
                    };
                    self.folder_cache.insert(folder_path.clone(), cache_entry);

                    self.assets = assets;
                    // Only change state to Assets if we were already in Assets state or if we want to switch
                    // For now, let's not automatically change state - keep current state
                    self.status_message = format!("Loaded {} assets", self.assets.len());

                    // Log successful command with success indicator
                    self.add_log_entry(format!(
                        "[{}] ✓ SUCCESS: {}",
                        Local::now().format("%H:%M:%S"),
                        self.last_executed_command
                    ));
                    self.command_in_progress = false; // Clear flag when command completes
                }
                Err(e) => {
                    self.status_message = format!("Error loading assets: {}", e);

                    // Log failed command with error indicator
                    self.add_log_entry(format!(
                        "[{}] ✗ ERROR: {} - {}",
                        Local::now().format("%H:%M:%S"),
                        self.last_executed_command,
                        e
                    ));
                    self.command_in_progress = false; // Clear flag when command completes
                }
            }
        } else {
            self.status_message = "No folder selected".to_string();
        }
    }

    pub async fn load_assets_for_selected_folder(&mut self) {
        if self.folders.is_empty() || self.selected_folder_index >= self.folders.len() {
            return; // No folders or invalid selection
        }

        let selected_folder = &self.folders[self.selected_folder_index];

        // Don't load assets for the parent directory indicator
        if selected_folder.uuid == ".." {
            self.assets.clear(); // Clear assets when selecting parent indicator
            return;
        }

        // Check if we have cached data for this folder
        if let Some(cached_data) = self.folder_cache.get(&selected_folder.path) {
            // Check if cache is still valid (less than 5 minutes old)
            if cached_data
                .timestamp
                .elapsed()
                .unwrap_or(std::time::Duration::MAX)
                < std::time::Duration::from_secs(300)
            {
                // 5 minutes
                self.assets = cached_data.assets.clone();
                self.status_message = format!(
                    "Loaded {} assets from cache for {}",
                    self.assets.len(),
                    selected_folder.name
                );
                self.last_executed_command = format!(
                    "pcli2 asset list --folder-path \"{}\" --format json --metadata",
                    selected_folder.path
                );
                self.command_history
                    .push(self.last_executed_command.clone());
                self.add_log_entry(format!(
                    "[{}] ✓ CACHED: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                return;
            }
        }

        // Set loading flag and status
        self.assets_loading_for_selection = true;
        self.last_executed_command = format!(
            "pcli2 asset list --folder-path \"{}\" --format json --metadata",
            selected_folder.path
        );
        self.command_history
            .push(self.last_executed_command.clone());
        self.command_in_progress = true; // Set flag when command starts
        self.status_message = format!("Loading assets for {}...", selected_folder.name);

        // Load assets in a separate task to avoid blocking the UI
        match pcli_commands::list_assets_in_folder(&selected_folder.path) {
            Ok(pcli_assets) => {
                // Convert pcli assets to our internal representation
                let assets: Vec<Asset> = pcli_assets
                    .into_iter()
                    .map(|a| Asset {
                        uuid: a.uuid,
                        name: a.name,
                        folder_uuid: selected_folder.path.clone(), // Use selected folder as parent
                        file_type: a.file_type,
                        size: a.file_size,
                        path: a.path,
                        metadata: a.metadata,
                    })
                    .collect();

                // Update or create cache entry with new asset data
                let cache_entry = FolderCache {
                    folders: self.folders.clone(), // Keep current folders in cache
                    assets: assets.clone(),
                    timestamp: std::time::SystemTime::now(),
                };
                self.folder_cache
                    .insert(selected_folder.path.clone(), cache_entry);

                self.assets = assets;
                self.status_message = format!(
                    "Loaded {} assets for {}",
                    self.assets.len(),
                    selected_folder.name
                );

                // Log successful command with success indicator
                self.add_log_entry(format!(
                    "[{}] ✓ SUCCESS: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
            Err(e) => {
                self.status_message =
                    format!("Error loading assets for {}: {}", selected_folder.name, e);

                // Log failed command with error indicator
                self.add_log_entry(format!(
                    "[{}] ✗ ERROR: {} - {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command,
                    e
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
        }

        // Clear loading flag
        self.assets_loading_for_selection = false;
    }

    async fn load_all_folders(&mut self) {
        let root_path = ""; // Use empty string to represent root

        // Check if we have cached data for root
        if let Some(cached_data) = self.folder_cache.get(root_path) {
            // Check if cache is still valid (less than 5 minutes old)
            if cached_data
                .timestamp
                .elapsed()
                .unwrap_or(std::time::Duration::MAX)
                < std::time::Duration::from_secs(300)
            {
                // 5 minutes
                self.folders = cached_data.folders.clone();
                self.status_message =
                    format!("Loaded {} top-level folders from cache", self.folders.len());
                self.last_executed_command = String::from("pcli2 folder list --format json");
                self.command_history
                    .push(self.last_executed_command.clone());
                self.add_log_entry(format!(
                    "[{}] ✓ CACHED: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                return;
            }
        }

        self.last_executed_command = String::from("pcli2 folder list --format json");
        self.command_history
            .push(self.last_executed_command.clone());
        self.command_in_progress = true; // Set flag when command starts
        self.status_message = "Loading all folders...".to_string();

        match pcli_commands::list_folders() {
            Ok(pcli_folders) => {
                // Convert pcli folders to our internal representation
                // Only include top-level folders (those without '/' in their path)
                let folders: Vec<Folder> = pcli_folders
                    .into_iter()
                    .filter(|f| !f.path.contains('/')) // Only top-level folders
                    .map(|f| Folder {
                        uuid: f.id, // Map 'id' from pcli to 'uuid' in our struct
                        name: f.name,
                        path: f.path, // Store the full path
                        folders_count: f.folders_count,
                        assets_count: f.assets_count,
                        parent_uuid: None, // pcli doesn't provide parent info in list
                        children: vec![],
                    })
                    .collect();

                // Cache the root folder data
                let cache_entry = FolderCache {
                    folders: folders.clone(),
                    assets: self.assets.clone(), // Keep current assets in cache
                    timestamp: std::time::SystemTime::now(),
                };
                self.folder_cache.insert(root_path.to_string(), cache_entry);

                self.folders = folders;
                self.status_message = format!("Loaded {} top-level folders", self.folders.len());

                // Log successful command with success indicator
                self.add_log_entry(format!(
                    "[{}] ✓ SUCCESS: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
            Err(e) => {
                self.status_message = format!("Error loading folders: {}", e);

                // Log failed command with error indicator
                self.add_log_entry(format!(
                    "[{}] ✗ ERROR: {} - {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command,
                    e
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
        }
    }

    pub async fn enter_folder(&mut self, folder_path: String) {
        // Store the folder name being entered so we can select it when going back
        let folder_name_entered = folder_path.split('/').next_back().unwrap_or(&folder_path).to_string();
        self.last_entered_folder_path = Some(folder_name_entered);

        let folder_path_clone = folder_path.clone();
        self.current_folder = Some(folder_path);

        // Force reload of folders by temporarily removing from cache
        self.folder_cache.remove(&folder_path_clone);
        self.load_folders_for_current_context().await;

        // Clear previous assets and load for the current folder
        self.assets.clear();
        self.load_assets_for_current_folder().await;

        // Reset selection indices when entering a new folder
        // If the first item is the parent directory indicator (".."), start selection from the next item
        if !self.folders.is_empty() && self.folders[0].uuid == ".." {
            self.selected_folder_index = 1;
        } else {
            self.selected_folder_index = 0;
        }
        self.selected_asset_index = 0;

        // Don't change the current state, just update the content
        // If we were in Folders state, stay there; if in Assets, stay there
    }

    pub async fn go_back_to_parent_folder(&mut self) {
        match &self.current_folder {
            Some(current_path) => {
                // Find the parent path by removing the last component
                if let Some(last_slash_idx) = current_path.rfind('/') {
                    let parent_path = current_path[..last_slash_idx].to_string();

                    // Extract the folder name that we're going back from
                    let folder_name_we_came_from = current_path[last_slash_idx + 1..].to_string();

                    self.current_folder = Some(parent_path);

                    // Reload both folders and assets for the new context
                    self.load_folders_for_current_context().await;
                    self.load_assets_for_current_folder().await;

                    // Find the index of the folder we came from in the current folder list
                    // First, try to find by name
                    if let Some(index) = self.folders.iter().position(|f| f.name == folder_name_we_came_from) {
                        self.selected_folder_index = index;
                    } else {
                        // If not found by name, try to find by UUID (in case folder name and UUID differ)
                        if let Some(index) = self.folders.iter().position(|f| f.uuid == folder_name_we_came_from) {
                            self.selected_folder_index = index;
                        } else {
                            // If still not found, default to the first item (or skip the parent indicator if present)
                            if !self.folders.is_empty() && self.folders[0].uuid == ".." {
                                self.selected_folder_index = 1;
                            } else {
                                self.selected_folder_index = 0;
                            }
                        }
                    }
                } else {
                    // If no slash, we're at a top-level folder, so go back to root
                    // Extract the folder name we're coming from
                    let folder_name_we_came_from = current_path.clone();

                    self.current_folder = None;

                    // Reload both folders and assets for the new context
                    self.load_folders_for_current_context().await;
                    self.load_assets_for_current_folder().await;

                    // Find the index of the folder we came from in the current folder list
                    if let Some(index) = self.folders.iter().position(|f| f.name == folder_name_we_came_from) {
                        self.selected_folder_index = index;
                    } else {
                        // If not found, default to the first item (or skip the parent indicator if present)
                        if !self.folders.is_empty() && self.folders[0].uuid == ".." {
                            self.selected_folder_index = 1;
                        } else {
                            self.selected_folder_index = 0;
                        }
                    }
                }

                // Stay in the same state but with updated content
                // If we were viewing assets before, continue viewing assets of the new folder
                // If we were viewing folders, continue viewing folders
            }
            None => {
                // Already at root, nothing to go back to
                self.status_message = "Already at root folder".to_string();
            }
        }
    }

    pub async fn switch_to_assets_view(&mut self) {
        if self.current_folder.is_some() {
            self.load_assets_for_current_folder().await;
            self.current_state = AppState::Assets;
        }
    }

    #[allow(dead_code)]
    pub async fn download_asset(&mut self, asset: &Asset) {
        self.status_message = format!("Downloading asset: {}...", asset.name);

        match pcli_commands::download_asset(&asset.uuid) {
            Ok(()) => {
                self.status_message = format!("Successfully downloaded: {}", asset.name);
            }
            Err(e) => {
                self.status_message = format!("Download failed: {}", e);
            }
        }
    }

    pub async fn download_asset_by_uuid(&mut self, asset_uuid: &str, asset_name: &str) {
        self.status_message = format!("Downloading asset: {}...", asset_name);

        match pcli_commands::download_asset(asset_uuid) {
            Ok(()) => {
                self.status_message = format!("Successfully downloaded: {}", asset_name);
            }
            Err(e) => {
                self.status_message = format!("Download failed: {}", e);
            }
        }
    }

    pub async fn upload_asset_interactive(&mut self) {
        // In a real implementation, this would open a file picker dialog
        // For now, we'll simulate with a placeholder
        self.status_message =
            "Upload feature: In a real implementation, this would open a file picker".to_string();
    }

    pub async fn perform_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.status_message = "Empty search query".to_string();
            return;
        }

        self.last_executed_command = format!(
            "pcli2 asset text-match --text \"{}\" --format json --metadata",
            self.search_query
        );
        self.command_history
            .push(self.last_executed_command.clone());
        self.command_in_progress = true; // Set flag when command starts
        self.status_message = format!("Searching for: {}", self.search_query);

        match pcli_commands::search_assets(&self.search_query) {
            Ok(pcli_assets) => {
                // Store search results separately from folder assets
                self.search_results = pcli_assets
                    .into_iter()
                    .map(|a| Asset {
                        uuid: a.uuid,
                        name: a.name,
                        folder_uuid: a.path.split('/').next().unwrap_or_default().to_string(), // Extract folder from path
                        file_type: a.file_type,
                        size: a.file_size,
                        path: a.path,
                        metadata: a.metadata,
                    })
                    .collect();

                self.status_message = format!("Found {} assets", self.search_results.len());

                // Log successful command with success indicator
                self.add_log_entry(format!(
                    "[{}] ✓ SUCCESS: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
            Err(e) => {
                self.status_message = format!("Search failed: {}", e);

                // Log failed command with error indicator
                self.add_log_entry(format!(
                    "[{}] ✗ ERROR: {} - {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command,
                    e
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
        }
    }

    #[allow(dead_code)]
    pub async fn upload_asset_to_current_folder(&mut self, file_path: &str) {
        if let Some(ref folder_path) = self.current_folder {
            self.status_message = format!("Uploading asset: {}...", file_path);

            match pcli_commands::upload_asset_to_folder(file_path, folder_path) {
                Ok(()) => {
                    self.status_message = format!("Successfully uploaded: {}", file_path);
                    // Reload assets to show the newly uploaded one
                    self.load_assets_for_current_folder().await;
                }
                Err(e) => {
                    self.status_message = format!("Upload failed: {}", e);
                }
            }
        } else {
            self.status_message = "No folder selected for upload".to_string();
        }
    }

    fn add_log_entry(&mut self, entry: String) {
        self.log_entries.push(entry);

        // Limit log history to 200 entries
        if self.log_entries.len() > 200 {
            // Remove oldest entries, keeping the most recent 200
            let excess = self.log_entries.len() - 200;
            self.log_entries.drain(0..excess);

            // Adjust scroll position if needed
            if self.log_scroll_position >= excess {
                self.log_scroll_position -= excess;
            } else {
                self.log_scroll_position = 0;
            }
        }

        // Always auto-scroll to the bottom to show the latest log entry
        self.log_scroll_position = self.log_entries.len().saturating_sub(1);
    }

    async fn handle_geometric_match_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Close the geometric match modal
                self.show_geometric_match_modal = false;
            }
            KeyCode::Up => {
                // Navigate up in geometric match results
                if !self.geometric_match_results.is_empty()
                    && self.geometric_match_scroll_position > 0 {
                    self.geometric_match_scroll_position -= 1;
                }
            }
            KeyCode::Down => {
                // Navigate down in geometric match results
                if !self.geometric_match_results.is_empty()
                    && self.geometric_match_scroll_position < self.geometric_match_results.len() - 1 {
                    self.geometric_match_scroll_position += 1;
                }
            }
            KeyCode::Left => {
                // Scroll left in the table (horizontal scrolling)
                if self.geometric_match_horizontal_scroll > 0 {
                    self.geometric_match_horizontal_scroll -= 1;
                }
            }
            KeyCode::Right => {
                // Scroll right in the table (horizontal scrolling)
                // We can't determine max columns without knowing the terminal width, so just increment
                self.geometric_match_horizontal_scroll += 1;
            }
            _ => {}
        }
    }
}

impl App {
    pub fn show_asset_details(&mut self) {
        if self.assets.is_empty() || self.selected_asset_index >= self.assets.len() {
            return; // No assets or invalid selection
        }

        let selected_asset = &self.assets[self.selected_asset_index];
        let asset_uuid = &selected_asset.uuid;

        self.last_executed_command = format!("pcli2 asset get --uuid \"{}\" --format json --metadata", asset_uuid);
        self.command_history.push(self.last_executed_command.clone());
        self.command_in_progress = true; // Set flag when command starts
        self.status_message = format!("Loading details for asset: {}", selected_asset.name);

        match pcli_commands::get_asset_details(asset_uuid) {
            Ok(pcli_asset_details) => {
                // Convert from pcli_commands::AssetDetails to app::AssetDetails
                let asset_details = crate::app::AssetDetails {
                    uuid: pcli_asset_details.uuid,
                    name: pcli_asset_details.name,
                    path: pcli_asset_details.path,
                    file_type: pcli_asset_details.file_type,
                    file_size: pcli_asset_details.file_size,
                    processing_status: pcli_asset_details.processing_status,
                    created_at: pcli_asset_details.created_at,
                    updated_at: pcli_asset_details.updated_at,
                    is_assembly: pcli_asset_details.is_assembly,
                    tenant_id: pcli_asset_details.tenant_id,
                    folder_id: pcli_asset_details.folder_id,
                    state: pcli_asset_details.state,
                    metadata: pcli_asset_details.metadata,
                };

                self.selected_asset_details = Some(asset_details);
                self.show_asset_details_modal = true;
                self.status_message = format!("Loaded details for {}", selected_asset.name);

                // Log successful command with success indicator
                self.add_log_entry(format!(
                    "[{}] ✓ SUCCESS: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
            Err(e) => {
                self.status_message = format!("Failed to load asset details: {}", e);

                // Log failed command with error indicator
                self.add_log_entry(format!(
                    "[{}] ✗ ERROR: {} - {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command,
                    e
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
        }
    }
    pub async fn perform_geometric_match(&mut self, asset_uuid: &str) {
        self.last_executed_command = format!(
            "pcli2 asset geometric-match --uuid \"{}\" --format json --metadata",
            asset_uuid
        );
        self.command_history
            .push(self.last_executed_command.clone());
        self.command_in_progress = true; // Set flag when command starts
        self.status_message = format!("Performing geometric match on asset: {}", asset_uuid);

        match pcli_commands::geometric_match(asset_uuid) {
            Ok(pcli_match_results) => {
                // Store geometric match results with similarity scores
                self.geometric_match_results = pcli_match_results
                    .into_iter()
                    .map(|match_entry| {
                        let asset = Asset {
                            uuid: match_entry.asset.uuid,
                            name: match_entry.asset.name,
                            folder_uuid: match_entry.asset.path.split('/').next().unwrap_or_default().to_string(), // Extract folder from path
                            file_type: match_entry.asset.file_type,
                            size: match_entry.asset.file_size,
                            path: match_entry.asset.path,
                            metadata: match_entry.asset.metadata,
                        };
                        (asset, match_entry.similarity_score)
                    })
                    .collect();

                self.status_message = format!("Found {} geometric matches", self.geometric_match_results.len());

                // Log successful command with success indicator
                self.add_log_entry(format!(
                    "[{}] ✓ SUCCESS: {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
            Err(e) => {
                self.status_message = format!("Geometric match failed: {}", e);

                // Log failed command with error indicator
                self.add_log_entry(format!(
                    "[{}] ✗ ERROR: {} - {}",
                    Local::now().format("%H:%M:%S"),
                    self.last_executed_command,
                    e
                ));
                self.command_in_progress = false; // Clear flag when command completes
            }
        }
    }
    pub async fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            crossterm::event::MouseEventKind::ScrollDown => {
                // Handle scrolling down in the active pane
                match self.active_pane {
                    crate::app::ActivePane::Folders => {
                        if !self.folders.is_empty() {
                            self.selected_folder_index =
                                (self.selected_folder_index + 1).min(self.folders.len() - 1);
                        }
                    }
                    crate::app::ActivePane::Assets => {
                        if !self.assets.is_empty() {
                            self.selected_asset_index =
                                (self.selected_asset_index + 1).min(self.assets.len() - 1);
                        }
                    }
                    crate::app::ActivePane::Log => {
                        // Scroll down in the log
                        if self.log_scroll_position < self.log_entries.len().saturating_sub(1) {
                            self.log_scroll_position += 1;
                        }
                    }
                }
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                // Handle scrolling up in the active pane
                match self.active_pane {
                    crate::app::ActivePane::Folders => {
                        if self.selected_folder_index > 0 {
                            self.selected_folder_index -= 1;
                        }
                    }
                    crate::app::ActivePane::Assets => {
                        if self.selected_asset_index > 0 {
                            self.selected_asset_index -= 1;
                        }
                    }
                    crate::app::ActivePane::Log => {
                        // Scroll up in the log
                        if self.log_scroll_position > 0 {
                            self.log_scroll_position -= 1;
                        }
                    }
                }
            }
            crossterm::event::MouseEventKind::Down(_) => {
                // Handle click events - could be extended to handle clicks on specific UI elements
                // For now, just handle scrolling based on which pane the mouse is in
            }
            _ => {}
        }
    }
}
