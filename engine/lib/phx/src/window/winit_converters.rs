use super::{CursorIcon, WindowLevel, WindowTheme};

pub fn convert_cursor_icon(cursor_icon: CursorIcon) -> winit::window::CursorIcon {
    match cursor_icon {
        CursorIcon::Default => winit::window::CursorIcon::Default,
        CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
        CursorIcon::Hand => winit::window::CursorIcon::Hand,
        CursorIcon::Arrow => winit::window::CursorIcon::Arrow,
        CursorIcon::Move => winit::window::CursorIcon::Move,
        CursorIcon::Text => winit::window::CursorIcon::Text,
        CursorIcon::Wait => winit::window::CursorIcon::Wait,
        CursorIcon::Help => winit::window::CursorIcon::Help,
        CursorIcon::Progress => winit::window::CursorIcon::Progress,
        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
        CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
        CursorIcon::Cell => winit::window::CursorIcon::Cell,
        CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
        CursorIcon::Alias => winit::window::CursorIcon::Alias,
        CursorIcon::Copy => winit::window::CursorIcon::Copy,
        CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
        CursorIcon::Grab => winit::window::CursorIcon::Grab,
        CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
        CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
        CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
        CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
        CursorIcon::EResize => winit::window::CursorIcon::EResize,
        CursorIcon::NResize => winit::window::CursorIcon::NResize,
        CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
        CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
        CursorIcon::SResize => winit::window::CursorIcon::SResize,
        CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
        CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
        CursorIcon::WResize => winit::window::CursorIcon::WResize,
        CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
        CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
        CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
        CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
        CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
        CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
    }
}

pub fn convert_window_level(window_level: WindowLevel) -> winit::window::WindowLevel {
    match window_level {
        WindowLevel::AlwaysOnBottom => winit::window::WindowLevel::AlwaysOnBottom,
        WindowLevel::Normal => winit::window::WindowLevel::Normal,
        WindowLevel::AlwaysOnTop => winit::window::WindowLevel::AlwaysOnTop,
    }
}

pub fn convert_winit_theme(theme: winit::window::Theme) -> WindowTheme {
    match theme {
        winit::window::Theme::Light => WindowTheme::Light,
        winit::window::Theme::Dark => WindowTheme::Dark,
    }
}

pub fn convert_window_theme(theme: WindowTheme) -> winit::window::Theme {
    match theme {
        WindowTheme::Light => winit::window::Theme::Light,
        WindowTheme::Dark => winit::window::Theme::Dark,
    }
}
