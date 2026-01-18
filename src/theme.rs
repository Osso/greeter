use iced::widget::{button, container, pick_list, text, text_input};
use iced::{Background, Border};

// Regreet color palette
pub mod colors {
    use iced::Color;

    pub const BACKGROUND: Color = Color::from_rgb(
        0x1e as f32 / 255.0,
        0x22 as f32 / 255.0,
        0x2a as f32 / 255.0,
    );

    pub const SURFACE: Color = Color::from_rgb(
        0x29 as f32 / 255.0,
        0x2e as f32 / 255.0,
        0x39 as f32 / 255.0,
    );

    pub const TEXT: Color = Color::WHITE;

    pub const TEXT_DIM: Color = Color::from_rgb(0.6, 0.6, 0.65);

    pub const ACCENT: Color = Color::from_rgb(
        0xe6 as f32 / 255.0,
        0xb4 as f32 / 255.0,
        0x50 as f32 / 255.0,
    );

    pub const ACCENT_HOVER: Color = Color::from_rgb(
        0xf0 as f32 / 255.0,
        0xc0 as f32 / 255.0,
        0x60 as f32 / 255.0,
    );

    pub const ACCENT_PRESSED: Color = Color::from_rgb(
        0xd0 as f32 / 255.0,
        0xa0 as f32 / 255.0,
        0x40 as f32 / 255.0,
    );

    pub const ACCENT_TEXT: Color = Color::from_rgb(
        0x0b as f32 / 255.0,
        0x0e as f32 / 255.0,
        0x14 as f32 / 255.0,
    );

    pub const BORDER: Color = Color::from_rgb(0.3, 0.32, 0.38);
}

pub fn background(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND)),
        text_color: Some(colors::TEXT),
        ..Default::default()
    }
}

pub fn card(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE)),
        text_color: Some(colors::TEXT),
        border: Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn text_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(colors::SURFACE),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: colors::TEXT_DIM,
        placeholder: colors::TEXT_DIM,
        value: colors::TEXT,
        selection: colors::ACCENT,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: colors::ACCENT,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: colors::ACCENT,
                width: 2.0,
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(colors::BACKGROUND),
            value: colors::TEXT_DIM,
            ..base
        },
    }
}

pub fn button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::ACCENT)),
        text_color: colors::ACCENT_TEXT,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::ACCENT_HOVER)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::ACCENT_PRESSED)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::SURFACE)),
            text_color: colors::TEXT_DIM,
            ..base
        },
    }
}

pub fn pick_list_style(_theme: &iced::Theme, status: pick_list::Status) -> pick_list::Style {
    let base = pick_list::Style {
        background: Background::Color(colors::SURFACE),
        text_color: colors::TEXT,
        placeholder_color: colors::TEXT_DIM,
        handle_color: colors::TEXT,
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
    };

    match status {
        pick_list::Status::Active => base,
        pick_list::Status::Hovered => pick_list::Style {
            border: Border {
                color: colors::ACCENT,
                ..base.border
            },
            ..base
        },
        pick_list::Status::Opened { .. } => pick_list::Style {
            border: Border {
                color: colors::ACCENT,
                width: 2.0,
                ..base.border
            },
            ..base
        },
    }
}

pub fn status_text(_theme: &iced::Theme) -> text::Style {
    text::Style {
        color: Some(colors::TEXT_DIM),
    }
}
