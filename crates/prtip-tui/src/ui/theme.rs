//! Color scheme and styling

use ratatui::style::{Color, Modifier, Style};

/// Header style (bold white)
pub fn header_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

/// Stage text style (cyan)
pub fn stage_style() -> Style {
    Style::default().fg(Color::Cyan)
}

/// Progress text style (green)
pub fn progress_style() -> Style {
    Style::default().fg(Color::Green)
}

/// Throughput text style (yellow)
pub fn throughput_style() -> Style {
    Style::default().fg(Color::Yellow)
}

/// Footer style (dim white)
pub fn footer_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::DIM)
}

/// Help screen style (white on blue background)
pub fn help_style() -> Style {
    Style::default().fg(Color::White).bg(Color::Blue)
}

/// Port status colors
pub fn port_open_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn port_closed_style() -> Style {
    Style::default().fg(Color::Red)
}

pub fn port_filtered_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn port_tls_style() -> Style {
    Style::default().fg(Color::Blue)
}
