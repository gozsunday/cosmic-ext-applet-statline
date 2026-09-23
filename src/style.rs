// SPDX-License-Identifier: MPL-2.0

//! Shared button styling for icon+text rows.
//!
//! This mirrors the `custom_row` language from the playbar reference:
//! full-width rows with list padding, 14px semibold text, and the desktop
//! global accent as the tint. Kept dependency-free (no palette crate).

use cosmic::iced::Color;
use cosmic::widget::button::Catalog;

// ---------------------------------------------------------------------------
// Contrast helper
// ---------------------------------------------------------------------------

/// Pick white or near-black text for readability on top of `background`.
///
/// Uses relative luminance with a 0.58 threshold (same value playbar uses).
#[must_use]
pub fn contrast_text(background: Color) -> Color {
    // Perceived brightness from sRGB coefficients.
    let luminance = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
    if luminance > 0.58 {
        // Bright accent -> dark text.
        Color::from_rgb8(17, 17, 17)
    } else {
        // Dark accent -> white text.
        Color::WHITE
    }
}

// ---------------------------------------------------------------------------
// Row class
// ---------------------------------------------------------------------------

/// List-row button class tinted with the desktop global accent.
///
/// - `selected = true` fills with `accent` and uses contrast text.
/// - `selected = false` keeps the native `ListItem` look.
///   Corner radii follow the active COSMIC theme.
#[allow(dead_code)]
#[must_use]
pub fn row_class(accent: Color, selected: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        // Resting state for the row.
        active: Box::new(move |focused, theme| {
            // Radius comes from desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_s;
            if selected {
                // Accent-filled row with readable text/icons.
                let mut s =
                    theme.active(focused, false, &cosmic::theme::Button::ListItem(radii));
                s.background = Some(cosmic::iced::Background::Color(accent));
                s.text_color = Some(contrast_text(accent));
                s.icon_color = Some(contrast_text(accent));
                s
            } else {
                // Native list row when not selected.
                theme.active(focused, false, &cosmic::theme::Button::ListItem(radii))
            }
        }),
        // Hover state for the row.
        hovered: Box::new(move |focused, theme| {
            // Radius comes from desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_s;
            if selected {
                // Keep accent fill on hover.
                let mut s =
                    theme.hovered(focused, false, &cosmic::theme::Button::ListItem(radii));
                s.background = Some(cosmic::iced::Background::Color(accent));
                s.text_color = Some(contrast_text(accent));
                s.icon_color = Some(contrast_text(accent));
                s
            } else {
                // Subtle neutral hover, never accent.
                theme.hovered(focused, false, &cosmic::theme::Button::ListItem(radii))
            }
        }),
        // Pressed state for the row.
        pressed: Box::new(move |focused, theme| {
            // Radius comes from desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_s;
            if selected {
                // Keep accent fill while pressed.
                let mut s =
                    theme.pressed(focused, false, &cosmic::theme::Button::ListItem(radii));
                s.background = Some(cosmic::iced::Background::Color(accent));
                s.text_color = Some(contrast_text(accent));
                s.icon_color = Some(contrast_text(accent));
                s
            } else {
                // Native pressed look when not selected.
                theme.pressed(focused, false, &cosmic::theme::Button::ListItem(radii))
            }
        }),
        // Disabled state for the row.
        disabled: Box::new(|theme| {
            // Radius comes from desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_s;
            theme.disabled(&cosmic::theme::Button::ListItem(radii))
        }),
    }
}

// ---------------------------------------------------------------------------
// Header pill class
// ---------------------------------------------------------------------------

/// Small header pill with a faint global-accent background.
///
/// Used for the left monitor button: external-link icon plus 13px medium text.
/// Background is accent at low alpha, text and icons use the full accent.
#[must_use]
pub fn header_pill_class(accent: Color) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        // Resting pill state.
        active: Box::new(move |focused, theme| {
            // Radius follows desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_m;
            // Base style from the neutral icon button.
            let mut s = theme.active(focused, false, &cosmic::theme::Button::Icon);
            // Rounded pill shape.
            s.border_radius = radii.into();
            // Faint accent wash so the pill reads as tinted.
            s.background = Some(cosmic::iced::Background::Color(Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.13,
            }));
            // Full accent for text and icons.
            s.text_color = Some(accent);
            s.icon_color = Some(accent);
            // No border for the header pill.
            s.border_width = 0.0;
            s
        }),
        // Hovered pill state.
        hovered: Box::new(move |focused, theme| {
            // Radius follows desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_m;
            // Base hovered style from the neutral icon button.
            let mut s = theme.hovered(focused, false, &cosmic::theme::Button::Icon);
            // Slightly stronger wash on hover.
            s.border_radius = radii.into();
            s.background = Some(cosmic::iced::Background::Color(Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.20,
            }));
            // Keep accent text and icons on hover.
            s.text_color = Some(accent);
            s.icon_color = Some(accent);
            s
        }),
        // Pressed pill state.
        pressed: Box::new(move |focused, theme| {
            // Radius follows desktop appearance settings.
            let radii = theme.cosmic().corner_radii.radius_m;
            // Base pressed style from the neutral icon button.
            let mut s = theme.pressed(focused, false, &cosmic::theme::Button::Icon);
            // Strongest wash while pressed.
            s.border_radius = radii.into();
            s.background = Some(cosmic::iced::Background::Color(Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.26,
            }));
            // Keep accent text and icons while pressed.
            s.text_color = Some(accent);
            s.icon_color = Some(accent);
            s
        }),
        // Disabled pill state.
        disabled: Box::new(|theme| theme.disabled(&cosmic::theme::Button::Icon)),
    }
}
