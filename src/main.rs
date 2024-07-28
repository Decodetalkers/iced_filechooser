mod dirfs;
mod icon_cache;
mod utils;
mod portal_option;

use std::path::{Path, PathBuf};
use dirfs::{update_dir_infos, DirUnit, FsInfo};
use iced::widget::{checkbox, scrollable, Column};
use iced::window::Id;
use iced::{executor, Length};
use iced::{Command, Element, Theme};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;
use iced_runtime::command::Action;
use iced_runtime::window::Action as WindowAction;

use iced_aw::{split, Split};

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

#[derive(Debug)]
struct FileChooser {
    dir: DirUnit,
    showhide: bool,
    preview_big_image: bool,
    selected_paths: Vec<PathBuf>,
    current_selected: Option<PathBuf>,
    right_spliter: Option<u16>,
    left_spliter: Option<u16>,
}

fn is_samedir(patha: &Path, pathb: &Path) -> bool {
    let Ok(origin_path) = patha.canonicalize() else {
        return false;
    };
    let Ok(self_path) = pathb.canonicalize() else {
        return false;
    };
    self_path.as_os_str() == origin_path.as_os_str()
}

#[derive(Debug, Clone)]
pub enum Message {
    RequestMutiSelect((bool, PathBuf)),
    RequestNextDirs((Vec<FsInfo>, PathBuf)),
    RequestSelect(PathBuf),
    RequestEnter(PathBuf),
    RequestShowHide(bool),
    RequestShowImage(bool),
    RequestAdjustRightSpliter(u16),
    RequestAdjustLeftSpliter(u16),
    SearchPatternCachedChanged(String),
    SearchPatternChanged,
    Confirm,
    Cancel,
}

impl Application for FileChooser {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                dir: DirUnit::enter(std::env::current_dir().unwrap().as_path()),
                showhide: false,
                preview_big_image: false,
                selected_paths: Vec::new(),
                current_selected: None,
                right_spliter: None,
                left_spliter: Some(400),
            },
            Command::perform(update_dir_infos("."), Message::RequestNextDirs),
        )
    }

    fn namespace(&self) -> String {
        String::from("Iced Filechooser")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RequestNextDirs((dirs, pathbuf)) => {
                if is_samedir(self.dir.current_dir(), &pathbuf) {
                    self.dir.append_infos(dirs);
                    self.dir.set_end();
                }
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
            Message::RequestMutiSelect((checked, file_path)) => {
                if checked {
                    if self.selected_paths.contains(&file_path) {
                        return Command::none();
                    }
                    self.selected_paths.push(file_path);
                } else {
                    let Some(index) = self.selected_paths.iter().position(|p| *p == file_path)
                    else {
                        return Command::none();
                    };
                    self.selected_paths.remove(index);
                }
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
            Message::RequestAdjustLeftSpliter(left_size) => {
                self.left_spliter = Some(left_size);
                Command::none()
            }
            Message::Cancel | Message::Confirm => {
                Command::single(Action::Window(WindowAction::Close(Id::MAIN)))
            }
        }
    }

    fn view(&self) -> Element<Message> {
        self.main_view()
    }
}

impl FileChooser {
    fn selected_view(&self) -> Element<Message> {
        let mut column = Column::new();
        for p in self.selected_paths.iter() {
            let rp = std::fs::canonicalize(p).unwrap();
            let name = rp.to_str().unwrap();
            column = column.push(
                checkbox(name, true).on_toggle(|_| Message::RequestMutiSelect((false, p.clone()))),
            );
        }
        scrollable(column).height(Length::Fill).into()
    }
    fn main_view(&self) -> Element<Message> {
        Split::new(
            self.selected_view(),
            self.dir.view(
                self.showhide,
                self.preview_big_image,
                self.right_spliter.as_ref(),
                self.current_selected.as_ref(),
                false,
                &self.selected_paths,
            ),
            self.left_spliter,
            split::Axis::Vertical,
            Message::RequestAdjustLeftSpliter,
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
