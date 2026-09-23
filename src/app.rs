// SPDX-License-Identifier: MPL-2.0

//! Statline applet: placeholder panel + popup shell with About/Settings.
//!
//! Plumbing follows the playbar reference (neutral theme only, no album
//! tint): per-run `AppModel`, rectangle-anchored popups, `watch_config`
//! hot-reload, and `Link`/accent styling from desktop settings.

use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Limits, Rectangle, Subscription, Vector};
use cosmic::prelude::*;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::surface::{action::LiveSettings, surface_task};
use cosmic::widget::{self, button, text};
use cosmic::Element;

use crate::config::Config;
use crate::fl;
use crate::popup;

// ---------------------------------------------------------------------------
// Popup navigation
// ---------------------------------------------------------------------------

/// Which popup page is currently shown.
///
/// - `Main` is the header row only for now (stats come later).
/// - `Settings` is title-only for now.
/// - `About` is the full COSMIC about widget.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopupPage {
    /// Main header with info/settings icons.
    #[default]
    Main,
    /// Settings placeholder page.
    Settings,
    /// Full about page.
    About,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

/// All applet state in one struct (iced calls `update` then `view`).
pub struct AppModel {
    /// COSMIC runtime state (panel size, theme, popup helpers).
    core: cosmic::Core,
    /// Open popup window id, or `None` when closed.
    popup: Option<Id>,
    /// Persistent user settings (RON files via cosmic-config).
    config: Config,
    /// Static about metadata shown on the About page.
    about: cosmic::widget::about::About,
    /// Current popup sub-page.
    page: PopupPage,
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Everything that can happen: clicks, popup close, config edits, links.
#[derive(Debug, Clone)]
pub enum Message {
    /// Panel pill clicked; carries click offset + button bounds for anchoring.
    TogglePopup(Vector, Rectangle),
    /// Popup window closed by the shell.
    PopupClosed(Id),
    /// Show the main header page.
    ShowMain,
    /// Show the settings title page.
    ShowSettings,
    /// Show the full about page.
    ShowAbout,
    /// Open a URL from the about page.
    OpenUrl(String),
    /// Launch COSMIC System Monitor (later: configured stats source).
    LaunchMonitor,
    /// RON config files changed on disk.
    UpdateConfig(Config),
}

// ---------------------------------------------------------------------------
// COSMIC application implementation
// ---------------------------------------------------------------------------

impl cosmic::Application for AppModel {
    /// Async executor for applet tasks.
    type Executor = cosmic::executor::Default;

    /// Startup flags (unused, always `()`).
    type Flags = ();

    /// Message type for `update`.
    type Message = Message;

    /// Reverse-DNS app id used for config + desktop files.
    const APP_ID: &'static str = "com.github.gozsunday.statline";

    /// Borrow the COSMIC core (required by the trait).
    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    /// Mutably borrow the COSMIC core (required by the trait).
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// One-time startup: load config and build about metadata.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Load persistent config, falling back to defaults on error.
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                // Config file parsed cleanly.
                Ok(config) => config,
                // Partial config: use what loaded, ignore errors for now.
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        // Build the static about page content once.
        let about = Self::statline_about();

        // Assemble the initial model with popup closed on Main.
        let app = AppModel {
            core,
            popup: None,
            config,
            about,
            page: PopupPage::Main,
        };

        // No startup task needed.
        (app, Task::none())
    }

    /// Shell asked to close the popup (e.g. clicked outside).
    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Panel content: placeholder text until sensor stats land.
    fn view(&self) -> Element<'_, Self::Message> {
        // Lowercase placeholder in 13px bold (normal body is 14px).
        let label = text("statline")
            .size(13)
            .font(cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::Bold,
                ..cosmic::font::default()
            });

        // Clickable pill that anchors the popup to its rectangle.
        let pill = button::custom(label)
            .padding([4, 10])
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_with_rectangle(Message::TogglePopup);

        // Let the panel autosize to the pill content.
        self.core.applet.autosize_window(pill).into()
    }

    /// Popup content: header / settings-title / about by page.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        // Global accent keeps header icons + back button in sync.
        let accent = popup::global_accent();
        popup::popup(&self.core, self.page, &self.about, accent)
    }

    /// Background subscriptions: config file hot-reload only for now.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Watch RON files so external edits update the applet live.
        Subscription::batch(vec![
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    /// Handle a message and optionally return an async task.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            // Navigate back to the main header page.
            Message::ShowMain => {
                // Reset navigation state; no async work needed.
                self.page = PopupPage::Main;
            }
            // Navigate to the settings title page.
            Message::ShowSettings => {
                // Reset navigation state; no async work needed.
                self.page = PopupPage::Settings;
            }
            // Navigate to the full about page.
            Message::ShowAbout => {
                // Show about content on next `view_window`.
                self.page = PopupPage::About;
            }
            // Open a link from the about page in the default browser.
            Message::OpenUrl(url) => {
                // Flatpak needs host spawn; native uses xdg-open directly.
                let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
                let res = if in_flatpak {
                    // Ask the host to open the URL outside the sandbox.
                    std::process::Command::new("flatpak-spawn")
                        .args(["--host", "xdg-open", &url])
                        .spawn()
                } else {
                    // Open directly on a native install.
                    std::process::Command::new("xdg-open").arg(&url).spawn()
                };
                // Log failures without crashing the applet.
                if let Err(e) = res {
                    eprintln!("open {url}: {e:?}");
                }
            }
            // Launch COSMIC System Monitor via desktop launcher.
            Message::LaunchMonitor => {
                // Desktop id + binary for COSMIC Monitor (see desktop file).
                const DESKTOP_ID: &str = "com.system76.CosmicMonitor";
                const FALLBACK_BIN: &str = "cosmic-monitor";
                // Inside Flatpak, launchers must run on the host.
                let in_flatpak = std::env::var("FLATPAK_ID").is_ok();
                // Try gtk4-launch, then gtk-launch, then the binary directly.
                let attempts: &[(&str, Vec<String>)] = &[
                    ("gtk4-launch", vec![DESKTOP_ID.to_owned()]),
                    ("gtk-launch", vec![DESKTOP_ID.to_owned()]),
                    (FALLBACK_BIN, vec![]),
                ];
                // Attempt each launcher until one spawns successfully.
                for (cmd, args) in attempts {
                    // Prefix with host spawn when sandboxed.
                    let res = if in_flatpak {
                        std::process::Command::new("flatpak-spawn")
                            .arg("--host")
                            .arg(cmd)
                            .args(args)
                            .spawn()
                    } else {
                        std::process::Command::new(cmd).args(args).spawn()
                    };
                    // Stop on first success; otherwise try the next fallback.
                    if res.is_ok() {
                        break;
                    }
                }
            }
            // Config files changed on disk; adopt the new values.
            Message::UpdateConfig(config) => {
                // Replace in-memory config with the reloaded one.
                self.config = config;
            }
            // Panel pill clicked: close if open, else open anchored popup.
            #[allow(clippy::cast_possible_truncation)]
            Message::TogglePopup(offset, bounds) => {
                // If a popup is already open, close it and reset nav.
                return if let Some(p) = self.popup.take() {
                    // Return to main so reopening starts fresh.
                    self.page = PopupPage::Main;
                    // Ask the shell to destroy the popup window.
                    surface_task(destroy_popup(p))
                } else {
                    // Allocate a fresh popup window id.
                    let new_id = Id::unique();
                    // Need the main window to anchor against; drop if missing.
                    let Some(parent) = self.core.main_window_id() else {
                        return Task::none();
                    };
                    // Open a blurred popup anchored to the panel button rect.
                    surface_task(app_popup(
                        // Frosted blur behind the popup content.
                        |_| LiveSettings {
                            blur: Some(true),
                            ..Default::default()
                        },
                        // Positioner places the popup under the pill.
                        move |state: &mut AppModel| {
                            // Base popup settings from the applet helper.
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                parent, new_id, None, None, None,
                            );
                            // Anchor to the clicked pill rectangle.
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            // Clamp popup size so pages stay readable (468 content + chrome).
                            popup_settings.positioner.size_limits = Limits::NONE
                                .max_width(480.0)
                                .min_width(300.0)
                                .min_height(200.0)
                                .max_height(880.0);
                            // Remember the open popup id.
                            state.popup = Some(new_id);
                            popup_settings
                        },
                        None,
                    ))
                };
            }
            // Shell closed the popup window; clear state if it matches.
            Message::PopupClosed(id) => {
                // Only clear when the closed id is our popup.
                if self.popup.as_ref() == Some(&id) {
                    // Forget the window and reset navigation.
                    self.popup = None;
                    self.page = PopupPage::Main;
                }
            }
        }
        // No async work for navigation/config messages.
        Task::none()
    }

    /// Default COSMIC applet styling for the panel + popup.
    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

// ---------------------------------------------------------------------------
// About metadata
// ---------------------------------------------------------------------------

impl AppModel {
    /// Static about content for the popup About page.
    fn statline_about() -> cosmic::widget::about::About {
        // Icon name matches the installed app id icon.
        const ICON: &str = "com.github.gozsunday.statline";
        // Upstream repo + issue tracker links.
        const REPO: &str = "https://github.com/gozsunday/cosmic-ext-applet-statline";
        const ISSUES: &str = "https://github.com/gozsunday/cosmic-ext-applet-statline/issues";
        // License shown in the about dialog.
        const LICENSE_URL: &str = "https://mozilla.org/MPL/2.0/";
        // Assemble the about dialog from fluent + cargo metadata.
        cosmic::widget::about::About::default()
            // Display name for the applet.
            .name(fl!("app-title"))
            // Panel/popup icon handle.
            .icon(widget::icon::from_name(ICON).handle())
            // Crate version from Cargo.toml.
            .version(env!("CARGO_PKG_VERSION"))
            // Author credit.
            .author("gozsunday")
            // Developer contact entry.
            .developers([("gozsunday", "gozmansunday@gmail.com")])
            // Repo + issue links (translated labels).
            .links([
                (fl!("links-main"), REPO),
                (fl!("links-issues"), ISSUES),
            ])
            // Short license identifier.
            .license("MPL-2.0-only")
            // Full license text URL.
            .license_url(LICENSE_URL)
            // One-line description from fluent strings.
            .comments(fl!("app-comment"))
    }
}
