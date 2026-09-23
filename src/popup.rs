// SPDX-License-Identifier: MPL-2.0

//! Popup pages for statline.
//!
//! Layout follows the playbar reference: a top header row on the main page
//! with info/settings icons right-aligned, plus About and Settings sub-pages
//! behind a `custom_row`-style back button tinted with the desktop global
//! accent (not album tint).

use cosmic::app::Core;
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Color, Length, Limits, Shadow};
use cosmic::widget::{self, autosize::autosize, button, icon, text, Row};
use cosmic::Element;

use std::sync::LazyLock;

use crate::app::{Message, PopupPage};
use crate::fl;
use crate::style;

// ---------------------------------------------------------------------------
// Popup dimensions
// ---------------------------------------------------------------------------

/// Fixed popup content width so the window stays stable.
const POPUP_CONTENT_WIDTH: f32 = 468.0;

/// Sub-page scroll cap: Settings/About scroll inside this height so the popup
/// stays compact. A bare scrollable is Shrink and would grow the window, so
/// it is wrapped in `container.max_height` (dialog precedent). Main is never
/// scrollable.
const SUBPAGE_SCROLL_MAX_HEIGHT: f32 = 720.0;

/// Wide popup autosize id (separate from libcosmic's 360px popup id).
static WIDE_AUTOSIZE_ID: LazyLock<cosmic::iced::id::Id> =
    LazyLock::new(|| cosmic::iced::id::Id::new("statline-popup-autosize"));

// ---------------------------------------------------------------------------
// Popup entry
// ---------------------------------------------------------------------------

/// Build the popup window for the current `page`.
///
/// - `Main` shows the header row only for now (stats/charts come later).
/// - `Settings` shows a back button plus the settings title.
/// - `About` shows a back button plus the COSMIC about widget.
pub fn popup<'a>(
    core: &'a Core,
    page: PopupPage,
    about: &'a cosmic::widget::about::About,
    accent: Color,
) -> Element<'a, Message> {
    // Spacing and outer padding follow the active COSMIC theme so the main
    // page sides always match the sub-pages (condensed vs normal panels).
    let spacing = cosmic::theme::spacing();
    let padding = if core.is_condensed() {
        // Compact panels use the small theme pad.
        spacing.space_s
    } else {
        // Normal panels use the medium theme pad.
        spacing.space_m
    };

    // Pick the inner page content for the current navigation state.
    let inner: Element<'a, Message> = match page {
        // Main page header row (monitor pill left, icons right).
        PopupPage::Main => main_header(core, accent),
        // Settings placeholder: title only for now.
        PopupPage::Settings => text::title3(fl!("settings")).into(),
        // Full about widget with clickable links.
        PopupPage::About => {
            widget::about(about, |url: &str| Message::OpenUrl(url.to_owned()))
        }
    };

    // Wrap sub-pages with a back button on top; main page has no back button.
    // Main is never scrollable; sub-pages scroll only past 640px.
    let content: Element<'a, Message> = match page {
        // Main page needs no back navigation and never scrolls.
        PopupPage::Main => inner,
        // Sub-pages get the accent back row fixed on top plus a scrolling page.
        PopupPage::Settings | PopupPage::About => {
            // Back button tinted with the global accent.
            let back = back_button(fl!("back"), Message::ShowMain, accent);
            widget::column::with_capacity(2)
                .push(
                    // Bottom gap separates nav from page content.
                    stack_pad(back, spacing.space_s),
                )
                .push(
                    // Scrollable page: Shrink until 640px, then scrolls.
                    // Wrapped in a max-height container so the window stays
                    // compact instead of growing with the content.
                    widget::container(widget::scrollable(stack_pad(inner, 0)))
                        .max_height(SUBPAGE_SCROLL_MAX_HEIGHT),
                )
                .spacing(spacing.space_xxs)
                .into()
        }
    };

    // Fix the popup width and apply the shared theme pad on all sides so
    // main and sub-pages stay aligned.
    let sized: Element<'a, Message> = widget::container(content)
        .width(Length::Fixed(POPUP_CONTENT_WIDTH))
        .padding(cosmic::iced::Padding::from(padding))
        .into();

    // Wide popup shell (not libcosmic's 360px `popup_container`).
    popup_container_wide(core, sized)
}

// ---------------------------------------------------------------------------
// Wide popup shell
// ---------------------------------------------------------------------------

/// Popup background wrapper without libcosmic's 360px width clamp.
///
/// `core.applet.popup_container` hardcodes autosize limits to exactly 360px,
/// so a 468px request shrinks back. This mirrors its look (rounded base bg,
/// 1px divider border, default shadow, anchor alignment) but measures with
/// wide limits matching the positioner (468..480 x 1..880).
fn popup_container_wide<'a>(
    core: &Core,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    // Align the shell toward the panel edge the applet sits on.
    let (vertical_align, horizontal_align) = match core.applet.anchor {
        // Vertical panels center vertically, stick to their side.
        PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
        PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
        // Horizontal panels stick to top/bottom, center horizontally.
        PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
        PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
    };

    // Inner background box with the COSMIC popup look.
    let background = widget::container(widget::container(content).style(|theme| {
        // Base surface color for the current transparency mode.
        let cosmic = theme.cosmic();
        let corners = cosmic.corner_radii;
        let bg = cosmic.background(theme.transparent).base;
        // Rounded card with divider border and default shadow.
        cosmic::iced::widget::container::Style {
            text_color: Some(cosmic.background(theme.transparent).on.into()),
            background: Some(Color::from(bg).into()),
            border: cosmic::iced::Border {
                radius: corners.radius_m.into(),
                width: 1.0,
                color: cosmic.background(theme.transparent).divider.into(),
            },
            shadow: Shadow::default(),
            icon_color: Some(cosmic.background(theme.transparent).on.into()),
            snap: true,
        }
    }))
    // Shrink to content height, aligned toward the panel.
    .height(Length::Shrink)
    .align_x(horizontal_align)
    .align_y(vertical_align);

    // Autosize notifies the shell of the measured size with wide limits.
    autosize(background, WIDE_AUTOSIZE_ID.clone())
        .limits(
            Limits::NONE
                .min_height(1.0)
                .min_width(POPUP_CONTENT_WIDTH)
                .max_width(480.0)
                .max_height(880.0),
        )
        .into()
}

// ---------------------------------------------------------------------------
// Small layout helper
// ---------------------------------------------------------------------------

/// Wrap a stacked sub-page element with only a bottom gap.
///
/// The shared theme pad on the outer popup container already handles sides,
/// so stacked rows only need vertical separation here.
fn stack_pad(element: Element<'_, Message>, bottom_gap: u16) -> Element<'_, Message> {
    // Only bottom gap; sides/top come from the outer popup container.
    let gap = f32::from(bottom_gap);
    widget::container(element)
        .padding([0.0, 0.0, gap, 0.0])
        .into()
}

// ---------------------------------------------------------------------------
// Main header row
// ---------------------------------------------------------------------------

/// Top section of the main page: monitor pill left, icons right.
///
/// Left pill shows an external-link icon plus `COSMIC System Monitor` text
/// and launches the monitor. It will later show the configured stats source.
/// Icons use the `Link` class so they follow the desktop global accent.
fn main_header(core: &Core, accent: Color) -> Element<'_, Message> {
    // Icon size follows the panel's suggested icon size.
    let size = core.applet.suggested_size(true);

    // Left source pill: bundled external-link glyph plus monitor name.
    let pill_content = Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        // Bundled libcosmic glyph (not a theme lookup) so it always renders.
        .push(widget::icon(widget::button::link::icon()).size(12))
        .push(
            // 13px Medium matches playbar's header pill text.
            text("COSMIC System Monitor")
                .size(13)
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Medium,
                    ..cosmic::font::default()
                }),
        );

    // Pill button with faint accent background, opens the system monitor.
    let pill = button::custom(pill_content)
        .padding([4, 10])
        .class(style::header_pill_class(accent))
        .on_press(Message::LaunchMonitor);

    // Info button opens the About page.
    let info = button::icon(icon::from_name("help-about-symbolic").size(size.0))
        .class(widget::button::ButtonClass::Link)
        .on_press(Message::ShowAbout);

    // Settings button opens the Settings page.
    let settings =
        button::icon(icon::from_name("preferences-system-symbolic").size(size.0))
            .class(widget::button::ButtonClass::Link)
            .on_press(Message::ShowSettings);

    // Left pill plus right-aligned icon cluster.
    Row::new()
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .push(pill)
        .push(text("").width(Length::Fill))
        .push(info)
        .push(settings)
        .into()
}

// ---------------------------------------------------------------------------
// Back button (custom_row style)
// ---------------------------------------------------------------------------

/// Back navigation pill styled exactly like the main-page monitor pill.
///
/// - Small `button::custom` (shrink, not full-width) with `go-previous` icon
///   plus text, mirroring the monitor pill's icon+text language.
/// - Same `12px` icon, `13px Medium` text, and `padding [4,10]`.
/// - Same faint global-accent wash via `style::header_pill_class`.
pub fn back_button(
    label: String,
    on_press: Message,
    accent: Color,
) -> Element<'static, Message> {
    // Icon + text share one row so sizes align optically.
    let content = Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        // 12px icon matches the monitor pill icon size.
        .push(icon::from_name("go-previous-symbolic").size(12).icon())
        .push(
            // 13px Medium matches the monitor pill text.
            text(label)
                .size(13)
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Medium,
                    ..cosmic::font::default()
                }),
        );

    // Faint accent pill, same class as the monitor button.
    button::custom(content)
        .padding([4, 10])
        .class(style::header_pill_class(accent))
        .on_press(on_press)
        .into()
}

/// Global accent color from desktop settings.
///
/// Central helper so header/back styling stays in sync with appearance.
#[must_use]
pub fn global_accent() -> Color {
    // `accent.base` is the user's COSMIC accent color.
    cosmic::theme::active().cosmic().accent.base.into()
}
