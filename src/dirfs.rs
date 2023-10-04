use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};
use std::{
    error::Error,
    fs::{self, DirEntry, Metadata},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

const DIR_ICON: &str = "text-directory";
const TEXT_ICON: &str = "text-plain";
use xdg_mime::SharedMimeInfo;

#[derive(Debug)]
pub enum FsInfo {
    File {
        icon: String,
        permission: [u32; 3],
        name: String,
    },
    Dir {
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
            Ok::<(String, Metadata), Box<dyn Error>>((file_name, metadata))
        })
        .flatten()
        .map(|(name, metadata)| {
            let permission = parse_permission(metadata.permissions().mode());
            if metadata.is_dir() {
                FsInfo::Dir { name, permission }
            } else {
                let mimeinfo = mime.get_mime_types_from_file_name(&name);
                let icon = mimeinfo
                    .first()
                    .and_then(|info| mime.lookup_generic_icon_name(info))
                    .unwrap_or(TEXT_ICON.to_string());
                FsInfo::File {
                    icon,
                    permission,
                    name,
                }
            }
        })
        .collect())
}

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
}
