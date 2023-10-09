mod dirfs;
mod utils;
use std::path::PathBuf;

use dirfs::{update_dir_infos, DirUnit, FsInfo};
use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

fn main() -> iced::Result {
    FileChooser::run(Settings::default())
}

#[derive(Debug)]
struct FileChooser {
    dir: DirUnit,
    showhide: bool,
    preview_big_image: bool,
    current_selected: Option<PathBuf>,
    right_spliter: Option<u16>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Check,
    RequestNextDirs(Vec<FsInfo>),
    RequestSelect(PathBuf),
    RequestEnter(PathBuf),
    RequestShowHide(bool),
    RequestShowImage(bool),
    RequestAdjustRightSpliter(u16),
    SearchPatternCachedChanged(String),
    SearchPatternChanged,
}

impl Application for FileChooser {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                dir: DirUnit::enter(&PathBuf::from(".")),
                showhide: false,
                preview_big_image: false,
                current_selected: None,
                right_spliter: None,
            },
            Command::perform(update_dir_infos("."), Message::RequestNextDirs),
        )
    }

    fn title(&self) -> String {
        String::from("Iced Filechooser")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RequestNextDirs(dirs) => {
                self.dir.append_infos(dirs);
                self.dir.set_end();
                Command::none()
            }
            Message::RequestEnter(path) => {
                self.dir = DirUnit::enter(&path.clone());
                Command::perform(update_dir_infos(path), Message::RequestNextDirs)
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
            Message::SearchPatternCachedChanged(pattern) => {
                self.dir.set_cache_pattern(&pattern);
                Command::none()
            }
            Message::SearchPatternChanged => {
                self.dir.set_pattern();
                Command::none()
            }
            Message::RequestAdjustRightSpliter(right_size) => {
                self.right_spliter = Some(right_size);
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        self.dir.view(
            self.showhide,
            self.preview_big_image,
            &self.right_spliter,
            &self.current_selected,
            false,
        )
    }
}
