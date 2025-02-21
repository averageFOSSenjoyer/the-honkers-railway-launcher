use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use anime_launcher_sdk::anime_game_core::prelude::*;
use anime_launcher_sdk::anime_game_core::star_rail::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;
use anime_launcher_sdk::star_rail::config::schema::launcher::LauncherStyle;

use crate::tr;

use super::general::*;
use super::enhancements::*;

pub static mut PREFERENCES_WINDOW: Option<adw::PreferencesWindow> = None;

pub struct PreferencesApp {
    general: AsyncController<GeneralApp>,
    enhancements: AsyncController<EnhancementsApp>
}

#[derive(Debug, Clone)]
pub enum PreferencesAppMsg {
    /// Supposed to be called automatically on app's run when the latest game version
    /// was retrieved from the API
    SetGameDiff(Option<VersionDiff>),

    /// Supposed to be called automatically on app's run when the latest main patch version
    /// was retrieved from remote repos
    SetMainPatch(Option<(Version, JadeitePatchStatusVariant)>),

    SetLauncherStyle(LauncherStyle),

    UpdateLauncherState,

    Toast {
        title: String,
        description: Option<String>
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for PreferencesApp {
    type Init = gtk::Window;
    type Input = PreferencesAppMsg;
    type Output = crate::ui::main::AppMsg;

    view! {
        preferences_window = adw::PreferencesWindow {
            set_title: Some(&tr!("preferences")),
            set_default_size: (700, 560),

            set_hide_on_close: true,
            set_modal: true,
            set_search_enabled: true,

            add = model.general.widget(),
            add = model.enhancements.widget(),

            connect_close_request[sender] => move |_| {
                if let Err(err) = Config::flush() {
                    sender.input(PreferencesAppMsg::Toast {
                        title: tr!("config-update-error"),
                        description: Some(err.to_string())
                    });
                }

                gtk::Inhibit::default()
            }
        }
    }

    async fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        tracing::info!("Initializing preferences window");

        let model = Self {
            general: GeneralApp::builder()
                .launch(())
                .forward(sender.input_sender(), std::convert::identity),

            enhancements: EnhancementsApp::builder()
                .launch(())
                .forward(sender.input_sender(), std::convert::identity)
        };

        let widgets = view_output!();

        widgets.preferences_window.set_transient_for(Some(&parent));

        unsafe {
            PREFERENCES_WINDOW = Some(widgets.preferences_window.clone());
        }

        #[allow(unused_must_use)] {
            model.enhancements.sender().send(EnhancementsAppMsg::SetGamescopeParent(widgets.preferences_window.clone()));

            model.general.sender().send(GeneralAppMsg::UpdateDownloadedWine);
            model.general.sender().send(GeneralAppMsg::UpdateDownloadedDxvk);
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        tracing::debug!("Called preferences window event: {:?}", msg);

        match msg {
            #[allow(unused_must_use)]
            PreferencesAppMsg::SetGameDiff(diff) => {
                self.general.sender().send(GeneralAppMsg::SetGameDiff(diff));
            }

            #[allow(unused_must_use)]
            PreferencesAppMsg::SetMainPatch(patch) => {
                self.general.sender().send(GeneralAppMsg::SetMainPatch(patch));
            }

            #[allow(unused_must_use)]
            PreferencesAppMsg::SetLauncherStyle(style) => {
                sender.output(Self::Output::SetLauncherStyle(style));
            }

            #[allow(unused_must_use)]
            PreferencesAppMsg::UpdateLauncherState => {
                sender.output(Self::Output::UpdateLauncherState {
                    perform_on_download_needed: false,
                    show_status_page: false
                });
            }

            PreferencesAppMsg::Toast { title, description } => unsafe {
                let toast = adw::Toast::new(&title);

                toast.set_timeout(4);

                if let Some(description) = description {
                    toast.set_button_label(Some(&tr!("details")));

                    let dialog = adw::MessageDialog::new(PREFERENCES_WINDOW.as_ref(), Some(&title), Some(&description));

                    dialog.add_response("close", &tr!("close", { "form" = "noun" }));
                    dialog.add_response("save", &tr!("save"));

                    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

                    dialog.connect_response(Some("save"), |_, _| {
                        if let Err(err) = open::that(crate::DEBUG_FILE.as_os_str()) {
                            tracing::error!("Failed to open debug file: {err}");
                        }
                    });

                    toast.connect_button_clicked(move |_| {
                        dialog.present();
                    });
                }

                PREFERENCES_WINDOW.as_ref().unwrap_unchecked().add_toast(toast);
            }
        }
    }
}
