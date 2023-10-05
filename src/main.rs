mod dirfs;

use std::path::PathBuf;

use dirfs::DirUnit;
use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

pub fn main() -> iced::Result {
    FileChooser::run(Settings::default())
}

#[derive(Debug)]
struct FileChooser {
    dir: DirUnit,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl Application for FileChooser {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                dir: DirUnit::new(&PathBuf::from("/")).unwrap(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Iced Filechooser")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        self.dir.view()
    }
}
