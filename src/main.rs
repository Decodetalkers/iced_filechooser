use iced_filechooser::portal_option::{FileChosen, FileFilter};
use iced_filechooser::FileChooser;

use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;

fn main() -> Result<(), iced_layershell::Error> {
    FileChooser::run(Settings {
        layer_settings: LayerShellSettings {
            margins: (200, 200, 200, 200),
            anchor: Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom,
            ..Default::default()
        },
        flags: FileChosen::OpenFile {
            handle_token: "abc".to_string(),
            accept_label: "a".to_string(),
            modal: true,
            multiple: false,
            directory: true,
            filters: [FileFilter::new("Svg Image").mimetype("image/svg+xml")].to_vec(),
            current_filter: None,
            choices: Vec::new(),
            current_folder: None,
        },
        ..Default::default()
    })
}
