use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use ruffle_frontend_utils::backends::navigator::NavigatorInterface;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use url::Url;
use winit::event_loop::EventLoopProxy;

use crate::custom_event::RuffleEvent;
use crate::gui::dialogs::network_access_dialog::{
    NetworkAccessDialogConfiguration, NetworkAccessDialogResult,
};
use crate::gui::DialogDescriptor;
use crate::util::open_url;

#[derive(Clone)]
pub struct DesktopNavigatorInterface {
    // Arc + Mutex due to macOS
    event_loop: Arc<Mutex<EventLoopProxy<RuffleEvent>>>,
}

impl DesktopNavigatorInterface {
    pub fn new(event_loop: EventLoopProxy<RuffleEvent>) -> Self {
        Self {
            event_loop: Arc::new(Mutex::new(event_loop)),
        }
    }
}

impl NavigatorInterface for DesktopNavigatorInterface {
    fn navigate_to_website(&self, url: Url, ask: bool) {
        if !ask {
            open_url(&url);
            return;
        }

        let _ = self
            .event_loop
            .lock()
            .expect("Non-poisoned event loop")
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::OpenUrl(url)));
    }

    fn open_file(&self, path: &Path) -> io::Result<File> {
        File::open(path).or_else(|e| {
            if cfg!(feature = "sandbox") {
                use rfd::FileDialog;
                let parent_path = path.parent().unwrap_or(path);

                if e.kind() == ErrorKind::PermissionDenied {
                    let attempt_sandbox_open = MessageDialog::new()
                        .set_level(MessageLevel::Warning)
                        .set_description(format!("The current movie is attempting to read files stored in {parent_path:?}.\n\nTo allow it to do so, click Yes, and then Open to grant read access to that directory.\n\nOtherwise, click No to deny access."))
                        .set_buttons(MessageButtons::YesNo)
                        .show() == MessageDialogResult::Yes;

                    if attempt_sandbox_open {
                        FileDialog::new().set_directory(parent_path).pick_folder();

                        return File::open(path);
                    }
                }
            }

            Err(e)
        })
    }

    async fn confirm_socket(&self, host: &str, port: u16) -> bool {
        let (notifier, receiver) = oneshot::channel();
        let _ = self
            .event_loop
            .lock()
            .expect("Non-poisoned event loop")
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::NetworkAccess(
                NetworkAccessDialogConfiguration::new(notifier, host, port),
            )));
        let result = receiver.await;
        result == Ok(NetworkAccessDialogResult::Allow)
    }
}
