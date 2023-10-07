use iced::theme;
use iced::widget::{button, container, scrollable, svg, text};
use iced::{Element, Length};
use libc::{S_IRUSR, S_IWUSR, S_IXUSR};
use std::{
    error::Error,
    fs::{self, Metadata},
    path::PathBuf,
};

use iced_aw::Grid;

static IMAGE: &[u8] = include_bytes!("../resources/text-plain.svg");
const DIR_ICON: &str = "text-directory";
const TEXT_ICON: &str = "text-plain";
use xdg_mime::SharedMimeInfo;

use crate::Message;

const COLUMN_WIDTH: f32 = 160.0;

const BUTTON_WIDTH: f32 = 150.0;

#[derive(Debug)]
pub struct DirUnit(Vec<FsInfo>);

impl DirUnit {
    pub fn view(&self, show_hide: bool) -> Element<Message> {
        let mut grid = Grid::with_column_width(COLUMN_WIDTH);
        for dir in self.fs_infos() {
            if !show_hide && dir.is_hidden() {
                continue;
            }
            grid = grid.push(dir.view());
        }

        scrollable(container(grid).center_x().width(Length::Fill)).into()
    }

    pub fn enter(dir: &PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self(ls_dir(dir)?))
    }

    fn fs_infos(&self) -> &Vec<FsInfo> {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum FsInfo {
    File {
        path: PathBuf,
        icon: String,
        permission: [u32; 3],
        name: String,
        symlink: Option<PathBuf>,
    },
    Dir {
        path: PathBuf,
        name: String,
        permission: [u32; 3],
        symlink: Option<PathBuf>,
    },
}

fn parse_permission(mode: u32) -> [u32; 3] {
    [mode & S_IRUSR, mode & S_IWUSR, mode & S_IXUSR]
}

pub fn ls_dir(dir: &PathBuf) -> Result<Vec<FsInfo>, Box<dyn Error>> {
    if !dir.is_dir() {
        return Err("Dir is not file".into());
    }
    let mime = SharedMimeInfo::new();

    Ok(fs::read_dir(dir)?
        .flatten()
        .flat_map(|file| {
            let file_name = file
                .file_name()
                .into_string()
                .map_err(|f| format!("Invalid entry: {:?}", f))?;
            let metadata = file.metadata()?;
            let path = file.path();
            Ok::<(String, PathBuf, Metadata), Box<dyn Error>>((file_name, path, metadata))
        })
        .map(|(name, path, metadata)| 'outputblock: {
            use std::os::unix::fs::MetadataExt;
            let permission = parse_permission(metadata.mode());
            if metadata.is_symlink() {
                let realpath = fs::read_link(&path).unwrap();
                if path.is_dir() {
                    break 'outputblock FsInfo::Dir {
                        path,
                        name,
                        permission,
                        symlink: Some(realpath),
                    };
                } else {
                    let mimeinfo = mime.get_mime_types_from_file_name(&name);
                    let icon = mimeinfo
                        .first()
                        .and_then(|info| mime.lookup_generic_icon_name(info))
                        .unwrap_or(TEXT_ICON.to_string());
                    break 'outputblock FsInfo::File {
                        path,
                        icon,
                        permission,
                        name,
                        symlink: Some(realpath),
                    };
                }
            }
            if metadata.is_dir() {
                FsInfo::Dir {
                    path,
                    name,
                    permission,
                    symlink: None,
                }
            } else {
                let mimeinfo = mime.get_mime_types_from_file_name(&name);
                let icon = mimeinfo
                    .first()
                    .and_then(|info| mime.lookup_generic_icon_name(info))
                    .unwrap_or(TEXT_ICON.to_string());
                FsInfo::File {
                    path,
                    icon,
                    permission,
                    name,
                    symlink: None,
                }
            }
        })
        .collect())
}

#[allow(unused)]
impl FsInfo {
    pub fn permission(&self) -> &[u32; 3] {
        match self {
            Self::Dir { permission, .. } => permission,
            Self::File { permission, .. } => permission,
        }
    }
    pub fn is_dir(&self) -> bool {
        matches!(self, FsInfo::Dir { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, FsInfo::File { .. })
    }

    pub fn is_readable(&self) -> bool {
        self.is_dir() && self.path().read_dir().is_ok()
    }

    pub fn is_writeable(&self) -> bool {
        let [_, w, _] = self.permission();
        *w == S_IWUSR
    }

    pub fn is_excutable(&self) -> bool {
        let [_, _, e] = self.permission();
        *e == S_IXUSR
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::File { icon, .. } => icon.as_str(),
            Self::Dir { .. } => DIR_ICON,
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            FsInfo::Dir { path, .. } => path.clone(),
            FsInfo::File { path, .. } => path.clone(),
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.name().starts_with('.')
    }

    pub fn is_symlink(&self) -> bool {
        match self {
            FsInfo::Dir { symlink, .. } => symlink.is_some(),
            FsInfo::File { symlink, .. } => symlink.is_some(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FsInfo::Dir { name, .. } => name,
            FsInfo::File { name, .. } => name,
        }
    }

    fn view(&self) -> Element<Message> {
        let handle = svg::Handle::from_memory(IMAGE);
        if self.is_dir() {
            let mut dirbtn = button(text(self.name()))
                .padding(10)
                .width(BUTTON_WIDTH)
                .height(BUTTON_WIDTH);
            if self.is_readable() {
                dirbtn = dirbtn.on_press(Message::RequestEnter(self.path()));
            }
            container(dirbtn)
                .height(COLUMN_WIDTH)
                .center_y()
                .center_x()
                .into()
        } else {
            container(
                button(svg(handle))
                    .padding(10)
                    .style(theme::Button::Positive)
                    .width(BUTTON_WIDTH)
                    .height(BUTTON_WIDTH),
            )
            .height(COLUMN_WIDTH)
            .center_y()
            .center_x()
            .into()
        }
    }
}
