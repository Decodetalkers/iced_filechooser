mod dirfs;
mod utils;
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
    showhide: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Check,
    RequestNext,
    RequestSelect,
    RequestEnter(PathBuf),
    RequestShowHide(bool),
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
                showhide: false,
            },
            Command::perform(async {}, |_| Message::RequestNext),
        )
    }

    fn title(&self) -> String {
        String::from("Iced Filechooser")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RequestNext => {
                self.dir.polldir().ok();
                if self.dir.ls_end() {
                    Command::none()
                } else {
                    Command::perform(async {}, |_| Message::RequestNext)
                }
            }
            Message::RequestEnter(path) => {
                if let Ok(dir) = DirUnit::enter(&path) {
                    self.dir = dir;
                    Command::perform(async {}, |_| Message::RequestNext)
                } else {
                    Command::none()
                }
            }
            Message::RequestShowHide(showhide) => {
                self.showhide = showhide;
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        self.dir.view(self.showhide, false)
    }
}
