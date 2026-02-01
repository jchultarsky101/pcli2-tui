# PCLI2-TUI

A terminal user interface (TUI) for the PCLI2 command-line tool, allowing intuitive navigation and management of Physna folders and assets.

## Features

- **Folder Navigation**: Browse Physna folder structure with a dual-panel interface
- **Asset Management**: List, view, and manage assets within folders
- **Search**: Search for assets across your Physna account
- **Upload/Download**: Upload new assets to folders and download existing assets
- **Intuitive Controls**: Easy keyboard navigation with clear status indicators

## Prerequisites

- Rust (1.70 or later)
- PCLI2 command-line tool installed and configured
- A terminal that supports raw mode (most modern terminals do)

## Installation

1. Ensure PCLI2 is installed and configured:
   ```bash
   pcli2 --help
   ```

2. Clone and build the project:
   ```bash
   git clone <repository-url>
   cd pcli2-tui
   cargo build --release
   ```

## Usage

Run the application:
```bash
cargo run
```

Or if built in release mode:
```bash
./target/release/pcli2-tui
```

### Keyboard Controls

- **Navigation**:
  - `j` or `↓` : Move down in list
  - `k` or `↑` : Move up in list
  - `Enter` : Enter folder or select asset
  - `q` or `Esc` : Quit application

- **Folder View**:
  - `a` : Switch to assets view for current folder
  - `/` : Enter search mode
  - `u` : Enter upload mode
  - `d` : Enter download mode

- **Asset View**:
  - `d` : Download selected asset
  - `q` : Return to folder view

- **Search Mode**:
  - Type to enter search query
  - `Enter` : Execute search
  - `Esc` : Cancel search

## Troubleshooting

### "Device not configured (os error 6)" Error

This error occurs when the application is run in an environment that doesn't support raw terminal mode (such as some IDE integrated terminals). To resolve this:

1. Run the application directly in a system terminal (Terminal.app, iTerm2, gnome-terminal, etc.)
2. Ensure you're running in a proper terminal environment

The application will work correctly when launched from a standard terminal.

## Architecture

The application is organized into three main modules:

- `app.rs`: Contains the application state and business logic
- `ui.rs`: Handles the rendering of the terminal user interface
- `pcli_commands.rs`: Interfaces with the PCLI2 command-line tool

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.