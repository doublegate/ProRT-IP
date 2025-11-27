//! FileBrowserWidget - Interactive file browser for importing target files
//!
//! The FileBrowserWidget provides a file system browser for selecting target import files:
//! - Directory navigation (up/down, into directories, parent navigation)
//! - File filtering (*.txt, *.csv, *.json, all files)
//! - Keyboard-driven interface (no mouse required)
//! - Error handling for permissions and I/O errors
//!
//! # Layout
//!
//! ```text
//! ┌─ File Browser ─────────────────────────────────────────────┐
//! │ Path: /home/user/targets                                    │
//! │ Filter: [Text Files] CSV  JSON  All                         │
//! │                                                              │
//! │ > [DIR] subdirectory/                                       │
//! │   [FILE] targets.txt (1.2 KB)                              │
//! │   [FILE] servers.csv (3.4 KB)                              │
//! │   [DIR] ../                                                 │
//! │                                                              │
//! │ 3 entries (1 dir, 2 files)                                  │
//! │ [↑/↓] Navigate [Enter] Select/Open [Backspace] Parent      │
//! │ [1-4] Filter [Esc] Cancel                                   │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - `↑`/`↓` - Navigate file list
//! - `Enter` - Select file or enter directory
//! - `Backspace` - Navigate to parent directory
//! - `1` - Filter: Text files (*.txt)
//! - `2` - Filter: CSV files (*.csv)
//! - `3` - Filter: JSON files (*.json)
//! - `4` - Filter: All files (*)
//! - `Esc` - Cancel and close browser
//! - `Home` - Jump to first entry
//! - `End` - Jump to last entry

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

/// File filter types for the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFilter {
    /// Text files (*.txt)
    TextFiles,
    /// CSV files (*.csv)
    CsvFiles,
    /// JSON files (*.json)
    JsonFiles,
    /// All files (*)
    All,
}

impl FileFilter {
    /// Check if a file name matches this filter
    pub fn matches(&self, filename: &str) -> bool {
        match self {
            FileFilter::TextFiles => filename.to_lowercase().ends_with(".txt"),
            FileFilter::CsvFiles => filename.to_lowercase().ends_with(".csv"),
            FileFilter::JsonFiles => filename.to_lowercase().ends_with(".json"),
            FileFilter::All => true,
        }
    }

    /// Get display name for filter
    pub fn display_name(&self) -> &'static str {
        match self {
            FileFilter::TextFiles => "Text Files",
            FileFilter::CsvFiles => "CSV Files",
            FileFilter::JsonFiles => "JSON Files",
            FileFilter::All => "All Files",
        }
    }
}

/// File system entry (file or directory)
#[derive(Debug, Clone)]
struct FsEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
    size: Option<u64>,
    is_parent: bool, // Special entry for "../"
}

impl FsEntry {
    /// Create from DirEntry
    fn from_dir_entry(entry: &DirEntry) -> io::Result<Self> {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };

        Ok(Self {
            path,
            name,
            is_dir,
            size,
            is_parent: false,
        })
    }

    /// Create parent entry "../"
    fn parent_entry(parent_path: PathBuf) -> Self {
        Self {
            path: parent_path,
            name: "../".to_string(),
            is_dir: true,
            size: None,
            is_parent: true,
        }
    }

    /// Format size for display
    fn format_size(&self) -> String {
        match self.size {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                }
            }
            None => "".to_string(),
        }
    }
}

/// FileBrowserWidget state
pub struct FileBrowserWidget {
    /// Current directory path
    current_path: PathBuf,

    /// Directory entries (files and subdirectories)
    entries: Vec<FsEntry>,

    /// Selected entry index
    selected_index: usize,

    /// Active file filter
    file_filter: FileFilter,

    /// Error message (if any)
    error_message: Option<String>,

    /// Whether the browser is visible
    visible: bool,
}

impl FileBrowserWidget {
    /// Create a new FileBrowserWidget starting at given path
    ///
    /// If path is invalid, defaults to current directory
    pub fn new(start_path: Option<PathBuf>) -> Self {
        let current_path = start_path
            .and_then(|p| p.canonicalize().ok())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));

        let mut widget = Self {
            current_path,
            entries: Vec::new(),
            selected_index: 0,
            file_filter: FileFilter::All,
            error_message: None,
            visible: false,
        };

        // Load initial directory
        if let Err(e) = widget.refresh_entries() {
            widget.error_message = Some(format!("Failed to read directory: {}", e));
        }

        widget
    }

    /// Show the file browser
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the file browser
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if browser is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Refresh directory entries with current filter
    fn refresh_entries(&mut self) -> io::Result<()> {
        self.error_message = None;
        self.entries.clear();

        // Add parent directory entry (if not at root)
        if let Some(parent) = self.current_path.parent() {
            self.entries
                .push(FsEntry::parent_entry(parent.to_path_buf()));
        }

        // Read directory entries
        let dir_entries = fs::read_dir(&self.current_path)?;

        for entry_result in dir_entries {
            match entry_result {
                Ok(entry) => {
                    match FsEntry::from_dir_entry(&entry) {
                        Ok(fs_entry) => {
                            // Include directories always, filter files based on active filter
                            if fs_entry.is_dir || self.file_filter.matches(&fs_entry.name) {
                                self.entries.push(fs_entry);
                            }
                        }
                        Err(_) => {
                            // Skip entries we can't read (permission issues, etc.)
                            continue;
                        }
                    }
                }
                Err(_) => {
                    // Skip invalid entries
                    continue;
                }
            }
        }

        // Sort: directories first, then files (alphabetically)
        self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        // Reset selection if out of bounds
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = 0;
        }

        Ok(())
    }

    /// Navigate into selected directory or select file
    pub fn select_current(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                // Navigate into directory
                self.current_path = entry.path.clone();
                if let Err(e) = self.refresh_entries() {
                    self.error_message = Some(format!("Failed to open directory: {}", e));
                }
                None
            } else {
                // File selected - return path
                Some(entry.path.clone())
            }
        } else {
            None
        }
    }

    /// Navigate to parent directory
    pub fn navigate_up(&mut self) -> io::Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Set file filter and refresh
    pub fn set_filter(&mut self, filter: FileFilter) {
        if self.file_filter != filter {
            self.file_filter = filter;
            if let Err(e) = self.refresh_entries() {
                self.error_message = Some(format!("Failed to apply filter: {}", e));
            }
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        }
    }

    /// Move to first entry
    pub fn move_to_first(&mut self) {
        self.selected_index = 0;
    }

    /// Move to last entry
    pub fn move_to_last(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }
    }

    /// Get current path
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Render the file browser widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Create main layout (path + filter + list + footer)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Path display
                Constraint::Length(3), // Filter buttons
                Constraint::Min(5),    // File list
                Constraint::Length(3), // Footer (help + stats)
            ])
            .split(area);

        // Render path
        self.render_path(frame, chunks[0]);

        // Render filter buttons
        self.render_filters(frame, chunks[1]);

        // Render file list
        self.render_file_list(frame, chunks[2]);

        // Render footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_path(&self, frame: &mut Frame, area: Rect) {
        let path_text = format!("Path: {}", self.current_path.to_string_lossy());

        let paragraph = Paragraph::new(path_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Current Directory "),
            )
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(paragraph, area);
    }

    fn render_filters(&self, frame: &mut Frame, area: Rect) {
        let filters = [
            ("1", FileFilter::TextFiles),
            ("2", FileFilter::CsvFiles),
            ("3", FileFilter::JsonFiles),
            ("4", FileFilter::All),
        ];

        let mut spans = vec![Span::raw("Filter: ")];

        for (i, (key, filter)) in filters.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }

            let style = if *filter == self.file_filter {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::Gray)
            };

            spans.push(Span::styled(
                format!("[{}] {}", key, filter.display_name()),
                style,
            ));
        }

        let paragraph =
            Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    fn render_file_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let is_selected = index == self.selected_index;

                let (prefix, name_color) = if entry.is_parent {
                    ("[DIR]", Color::Yellow)
                } else if entry.is_dir {
                    ("[DIR]", Color::Blue)
                } else {
                    ("[FILE]", Color::White)
                };

                let size_str = if !entry.is_dir {
                    format!(" ({})", entry.format_size())
                } else {
                    String::new()
                };

                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::raw(if is_selected { "> " } else { "  " }),
                    Span::styled(prefix, Style::default().fg(name_color)),
                    Span::raw(" "),
                    Span::styled(&entry.name, style.fg(name_color)),
                    Span::styled(size_str, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let title = if let Some(ref err) = self.error_message {
            format!(" File List - ERROR: {} ", err)
        } else {
            " File List ".to_string()
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(title),
        );

        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let dir_count = self.entries.iter().filter(|e| e.is_dir).count();
        let file_count = self.entries.iter().filter(|e| !e.is_dir).count();

        let lines = vec![
            Line::from(format!(
                "{} entries ({} dirs, {} files)",
                self.entries.len(),
                dir_count,
                file_count
            )),
            Line::from(
                "[↑/↓] Navigate [Enter] Select/Open [Backspace] Parent [1-4] Filter [Esc] Cancel",
            ),
        ];

        let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::Gray));

        frame.render_widget(paragraph, area);
    }

    /// Handle keyboard event
    ///
    /// Returns Some(PathBuf) if file was selected, None otherwise
    pub fn handle_event(&mut self, event: Event) -> Option<PathBuf> {
        if !self.visible {
            return None;
        }

        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up => {
                    self.move_up();
                    None
                }
                KeyCode::Down => {
                    self.move_down();
                    None
                }
                KeyCode::Home => {
                    self.move_to_first();
                    None
                }
                KeyCode::End => {
                    self.move_to_last();
                    None
                }
                KeyCode::Enter => {
                    // Select current (enter dir or select file)
                    let result = self.select_current();
                    if result.is_some() {
                        self.hide();
                    }
                    result
                }
                KeyCode::Backspace => {
                    let _ = self.navigate_up();
                    None
                }
                KeyCode::Char('1') => {
                    self.set_filter(FileFilter::TextFiles);
                    None
                }
                KeyCode::Char('2') => {
                    self.set_filter(FileFilter::CsvFiles);
                    None
                }
                KeyCode::Char('3') => {
                    self.set_filter(FileFilter::JsonFiles);
                    None
                }
                KeyCode::Char('4') => {
                    self.set_filter(FileFilter::All);
                    None
                }
                KeyCode::Esc => {
                    self.hide();
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Default for FileBrowserWidget {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        File::create(temp_path.join("test.txt"))
            .unwrap()
            .write_all(b"test content")
            .unwrap();
        File::create(temp_path.join("data.csv"))
            .unwrap()
            .write_all(b"a,b,c\n1,2,3")
            .unwrap();
        File::create(temp_path.join("config.json"))
            .unwrap()
            .write_all(b"{}")
            .unwrap();
        File::create(temp_path.join("readme.md"))
            .unwrap()
            .write_all(b"# README")
            .unwrap();

        // Create subdirectory
        fs::create_dir(temp_path.join("subdir")).unwrap();
        File::create(temp_path.join("subdir/nested.txt"))
            .unwrap()
            .write_all(b"nested")
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_file_filter_matches() {
        assert!(FileFilter::TextFiles.matches("test.txt"));
        assert!(FileFilter::TextFiles.matches("TEST.TXT")); // Case insensitive
        assert!(!FileFilter::TextFiles.matches("test.csv"));

        assert!(FileFilter::CsvFiles.matches("data.csv"));
        assert!(!FileFilter::CsvFiles.matches("data.txt"));

        assert!(FileFilter::JsonFiles.matches("config.json"));
        assert!(!FileFilter::JsonFiles.matches("config.xml"));

        assert!(FileFilter::All.matches("anything.xyz"));
    }

    #[test]
    fn test_file_browser_new() {
        let temp_dir = create_test_dir();
        let browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        assert_eq!(
            browser.current_path(),
            temp_dir.path().canonicalize().unwrap()
        );
        assert!(!browser.is_visible());
        assert_eq!(browser.file_filter, FileFilter::All);
    }

    #[test]
    fn test_file_browser_refresh_all_files() {
        let temp_dir = create_test_dir();
        let browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        // Should see all files + subdir + parent (..)
        assert!(browser.entries.len() >= 5); // parent + subdir + 4 files
    }

    #[test]
    fn test_file_browser_filter_text_files() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        browser.set_filter(FileFilter::TextFiles);

        // Should see: parent (..), subdir, test.txt (1 txt file)
        let txt_files: Vec<_> = browser
            .entries
            .iter()
            .filter(|e| !e.is_dir && e.name.ends_with(".txt"))
            .collect();
        assert_eq!(txt_files.len(), 1);
        assert_eq!(txt_files[0].name, "test.txt");
    }

    #[test]
    fn test_file_browser_filter_csv_files() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        browser.set_filter(FileFilter::CsvFiles);

        let csv_files: Vec<_> = browser
            .entries
            .iter()
            .filter(|e| !e.is_dir && e.name.ends_with(".csv"))
            .collect();
        assert_eq!(csv_files.len(), 1);
        assert_eq!(csv_files[0].name, "data.csv");
    }

    #[test]
    fn test_file_browser_navigation_up_down() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        assert_eq!(browser.selected_index, 0);

        browser.move_down();
        assert_eq!(browser.selected_index, 1);

        browser.move_down();
        assert_eq!(browser.selected_index, 2);

        browser.move_up();
        assert_eq!(browser.selected_index, 1);

        browser.move_up();
        assert_eq!(browser.selected_index, 0);

        // At boundary - should stay at 0
        browser.move_up();
        assert_eq!(browser.selected_index, 0);
    }

    #[test]
    fn test_file_browser_navigate_into_directory() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        // Find subdir entry
        let subdir_index = browser
            .entries
            .iter()
            .position(|e| e.name == "subdir" && e.is_dir)
            .unwrap();

        browser.selected_index = subdir_index;
        let result = browser.select_current();

        // Should navigate into subdir, not return path
        assert!(result.is_none());
        assert!(browser.current_path().ends_with("subdir"));

        // Should see nested.txt
        let has_nested = browser.entries.iter().any(|e| e.name == "nested.txt");
        assert!(has_nested);
    }

    #[test]
    fn test_file_browser_select_file() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        // Find test.txt
        let txt_index = browser
            .entries
            .iter()
            .position(|e| e.name == "test.txt")
            .unwrap();

        browser.selected_index = txt_index;
        let result = browser.select_current();

        // Should return file path
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.ends_with("test.txt"));
    }

    #[test]
    fn test_file_browser_navigate_to_parent() {
        let temp_dir = create_test_dir();
        let mut browser = FileBrowserWidget::new(Some(temp_dir.path().to_path_buf()));

        // Navigate into subdir
        let subdir_index = browser
            .entries
            .iter()
            .position(|e| e.name == "subdir")
            .unwrap();
        browser.selected_index = subdir_index;
        browser.select_current();

        assert!(browser.current_path().ends_with("subdir"));

        // Navigate back to parent
        browser.navigate_up().unwrap();
        assert_eq!(
            browser.current_path(),
            temp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_file_browser_show_hide() {
        let browser = FileBrowserWidget::default();
        assert!(!browser.is_visible());

        let mut browser = browser;
        browser.show();
        assert!(browser.is_visible());

        browser.hide();
        assert!(!browser.is_visible());
    }
}
