# Recent & Prefetch Manager

**A GUI utility for Windows that monitors and manages Recent folder tracking and the SysMain (Prefetch) service.**

## 🎯 Features

### 📁 Recent Folder Tracking

- Check if Recent files tracking is enabled in Windows Explorer
- View count of `.lnk` files and timestamps (oldest/newest)
- Enable tracking via registry keys:
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Start_TrackDocs`
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowRecent`
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowFrequent`
- Open Recent folder directly from the interface

### ⚙️ SysMain Service (Prefetch)

- Check SysMain service status (running/stopped)
- View startup type (automatic/manual/disabled)
- Count Prefetch `.pf` files in `C:\Windows\Prefetch`
- Enable and start SysMain service (requires administrator privileges)
- View oldest and newest Prefetch file timestamps
- Open Prefetch folder directly from the interface

### 🔒 Permissions

- Non-admin mode: View Recent status and file counts
- Admin mode: Full control over SysMain service and Prefetch access
- One-click "Restart as Administrator" button when elevated privileges are needed

## 🚀 Installation

### Requirements

- Windows 10/11
- Rust toolchain (for building from source)

### Simplest way

```powershell
cargo run
```

### Build from Source

```powershell
cargo build --release
```

The executable will be located at `target\release\recent-enabler.exe`.

### Run

Simply launch the executable:

```powershell
.\target\release\recent-enabler.exe
```

Or use the included batch file for administrator mode:

```powershell
.\run_as_admin.bat
```

## 📋 Usage

The application launches in GUI mode with a dark theme. The interface is divided into two main cards:

### Recent Card

- Shows current tracking status (enabled/disabled)
- Displays file count and timestamps
- "Enable Recent Tracking" button appears when disabled
- "Open Folder" button to open the Recent folder in Explorer

### Prefetch Card

- Shows SysMain service status and startup type
- Displays Prefetch file count and timestamps
- "Enable Prefetch Service" button (requires admin)
- "Open Folder" button to open the Prefetch folder in Explorer

### Buttons

- **Refresh**: Reload all status information
- **Restart as Administrator**: Relaunch the app with elevated privileges
- **Open Folder**: Open the respective folder in Windows Explorer

## 🔍 What It Checks

### Recent Tracking

1. **Folder Path**: `%APPDATA%\Microsoft\Windows\Recent`
2. **Registry Keys**:
   - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Start_TrackDocs`
   - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowRecent`
   - `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowFrequent`
3. **File Count**: Number of `.lnk` files
4. **Timestamps**: Oldest and newest file modification times

### SysMain Service

1. **Service Status**: Running, Stopped, Paused, or Unknown
2. **Startup Type**: Automatic, Manual, Disabled, or Unknown
3. **Prefetch Folder**: `C:\Windows\Prefetch`
4. **File Count**: Number of `.pf` files
5. **Timestamps**: Oldest and newest Prefetch file modification times

## ⚠️ Important Notes

- **Administrator Privileges** are required for:
  - Starting/stopping the SysMain service
  - Changing service startup type
  - Reading Prefetch folder contents (on some systems)
- **Windows Only**: This utility is designed specifically for Windows 10/11
- Error messages are displayed at the top of the window when operations fail

## 🏗️ Architecture

The project is organized into modules:

```
src/
├── main.rs      Entry point and window configuration
├── app.rs       Application state, messages, and logic
├── ui.rs        Reusable UI components and styling
├── recent.rs    Recent folder operations and registry handling
├── sysmain.rs   SysMain service control and Prefetch operations
└── utils.rs     Utility functions (admin detection)
```

## 🔧 Technical Details

### Registry Operations

The utility manipulates the following Windows Registry keys:

- `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Start_TrackDocs`

  - `0` = Tracking disabled
  - `1` = Tracking enabled

- `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowRecent`

  - `0` = Hide recent items in Explorer
  - `1` = Show recent items

- `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\ShowFrequent`
  - `0` = Hide frequent items in Explorer
  - `1` = Show frequent items

### Service Management

Uses Windows Service Control Manager API to:

- Query service status
- Read startup configuration
- Modify startup type
- Start/stop the SysMain service
