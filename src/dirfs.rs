use iced::widget::{button, column, row, scrollable, text};
use iced::Element;
use libc::{S_IRUSR, S_IWUSR, S_IXUSR};
use std::{
    error::Error,
    fs::{self, Metadata},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

const DIR_ICON: &str = "text-directory";
const TEXT_ICON: &str = "text-plain";
use xdg_mime::SharedMimeInfo;

use crate::Message;

#[derive(Debug)]
pub struct DirUnit(Vec<FsInfo>);

impl DirUnit {
    pub fn view(&self) -> Element<Message> {
        let mut rows: Vec<Element<Message>> = vec![];
        for dir in self.0.chunks_exact(4) {
            let mut elements: Vec<Element<Message>> = vec![];
            for unit in dir {
                elements.push(button(text(unit.name())).into());
            }
            let row = row(elements);
            rows.push(row.into());
        }
        scrollable(column(rows)).into()
    }

    pub fn new(dir: &PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self(ls_dir(dir)?))
    }
}

#[derive(Debug, Clone)]
pub enum FsInfo {
    File {
        path: PathBuf,
        icon: String,
        permission: [u32; 3],
        name: String,
    },
    Dir {
        path: PathBuf,
        name: String,
        permission: [u32; 3],
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
        .into_iter()
        .map(|file| {
            let file_name = file
                .file_name()
                .into_string()
                .or_else(|f| Err(format!("Invalid entry: {:?}", f)))?;
            let metadata = file.metadata()?;
            let path = file.path();
            Ok::<(String, PathBuf, Metadata), Box<dyn Error>>((file_name, path, metadata))
        })
        .flatten()
        .map(|(name, path, metadata)| {
            let permission = parse_permission(metadata.permissions().mode());
            if metadata.is_dir() {
                FsInfo::Dir {
                    path,
                    name,
                    permission,
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
        let [r, _, _] = self.permission();
        *r != 0
    }

    pub fn is_writeable(&self) -> bool {
        let [_, w, _] = self.permission();
        *w != 0
    }

    pub fn is_excutable(&self) -> bool {
        let [_, _, e] = self.permission();
        *e != 0
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::File { icon, .. } => icon.as_str(),
            Self::Dir { .. } => DIR_ICON,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.icon().starts_with(".")
    }

    pub fn name(&self) -> &str {
        match self {
            FsInfo::Dir { name, .. } => &name,
            FsInfo::File { name, .. } => &name,
        }
    }
}
