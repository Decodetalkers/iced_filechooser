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
    preview_big_image: bool,
    current_selected: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Check,
    RequestNext,
    RequestSelect(PathBuf),
    RequestEnter(PathBuf),
    RequestShowHide(bool),
    RequestShowImage(bool),
}

impl Application for FileChooser {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                dir: DirUnit::enter(&PathBuf::from(".")).unwrap(),
                showhide: false,
                preview_big_image: false,
                current_selected: None,
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
            Message::RequestShowImage(showimage) => {
                self.preview_big_image = showimage;
                Command::none()
            }
            Message::RequestSelect(path) => {
                if self.current_selected.clone().is_some_and(|p| {
                    p.canonicalize().unwrap().as_os_str()
                        == path.canonicalize().unwrap().as_os_str()
                }) {
                    self.current_selected = None;
                } else {
                    self.current_selected = Some(path);
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        self.dir.view(
            self.showhide,
            self.preview_big_image,
            &self.current_selected,
            false,
        )
    }
}
