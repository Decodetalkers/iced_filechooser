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
        ..Default::default()
    })
}
