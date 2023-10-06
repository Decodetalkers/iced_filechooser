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

#[derive(Debug, Clone)]
pub enum Message {
    RequestEnter(PathBuf),
}

impl Application for FileChooser {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                dir: DirUnit::enter(&PathBuf::from("/")).unwrap(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Iced Filechooser")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let Message::RequestEnter(path) = message;
        if let Ok(dir) = DirUnit::enter(&path) {
            self.dir = dir;
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        self.dir.view(false)
    }
}
