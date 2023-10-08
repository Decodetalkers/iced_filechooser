use iced::alignment;
use iced::widget::{button, checkbox, column, container, image, row, scrollable, svg, text};
use iced::{theme, Element, Length};
use libc::{S_IRUSR, S_IWUSR, S_IXUSR};
use std::fs::ReadDir;
use std::str::FromStr;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use iced_aw::Grid;

use crate::utils::get_icon;

use mime::Mime;
use xdg_mime::SharedMimeInfo;

use once_cell::sync::Lazy;

static MIME: Lazy<SharedMimeInfo> = Lazy::new(SharedMimeInfo::new);

static TEXT_IMAGE: &[u8] = include_bytes!("../resources/text-plain.svg");

static DIR_IMAGE: &[u8] = include_bytes!("../resources/inode-directory.svg");

static GO_PREVIOUS: &[u8] = include_bytes!("../resources/go-previous.svg");

const DIR_ICON: &str = "inode-directory";
const TEXT_ICON: &str = "text-plain";

use crate::Message;

const COLUMN_WIDTH: f32 = 200.0;

const BUTTON_WIDTH: f32 = 170.0;

#[derive(Debug)]
pub struct DirUnit {
    is_end: bool,
    iter: std::iter::Flatten<ReadDir>,
    infos: Vec<FsInfo>,
    current_dir: PathBuf,
}

fn get_dir_name(dir: &Path) -> String {
    let mut output = dir
        .to_string_lossy()
        .to_string()
        .split('/')
        .last()
        .unwrap_or("/")
        .to_string();
    if output.is_empty() {
        output = "/".to_string();
    }
    output
}

impl DirUnit {
    fn get_parent_path(&self) -> Option<PathBuf> {
        self.current_dir.parent().map(|path| path.into())
    }

    fn get_prevouse_icon(&self) -> Element<Message> {
        if let Some(icon) = get_icon("Adwaita", "go-previous") {
            return svg(svg::Handle::from_path(icon))
                .width(20)
                .height(20)
                .into();
        }
        svg(svg::Handle::from_memory(GO_PREVIOUS))
            .width(20)
            .height(20)
            .into()
    }

    pub fn view(&self, show_hide: bool, preview_image: bool, select_dir: bool) -> Element<Message> {
        let mut grid = Grid::with_column_width(COLUMN_WIDTH);
        let filter_way = |dir: &&FsInfo| show_hide || !dir.is_hidden();
        for dir in self.fs_infos().iter().filter(filter_way) {
            grid = grid.push(dir.view(select_dir, preview_image));
        }
        //let mainview = column![grid, self.title_bar(show_hide)];
        let bottom = scrollable(container(grid).center_x().width(Length::Fill));
        column![self.title_bar(show_hide, preview_image), bottom]
            .spacing(10)
            .into()
    }

    fn title_bar(&self, show_hide: bool, preview_image: bool) -> Element<Message> {
        let current_dir = fs::canonicalize(&self.current_dir).unwrap();
        let mut rowvec: Vec<Element<Message>> = Vec::new();
        if let Some(parent) = self.get_parent_path() {
            let btn: Element<Message> = button(self.get_prevouse_icon())
                .style(theme::Button::Secondary)
                .on_press(Message::RequestEnter(parent))
                .into();
            rowvec.push(btn);
        }

        let mut dirbtn: Vec<Element<Message>> = Vec::new();

        let mut current_path_dir = current_dir.clone();

        dirbtn.push(
            button(text(get_dir_name(&current_path_dir)))
                .on_press(Message::RequestEnter(current_path_dir.clone()))
                .into(),
        );

        while let Some(parent) = current_path_dir.parent() {
            current_path_dir = PathBuf::from(parent);
            let mut newbtns = vec![button(text(get_dir_name(&current_path_dir)))
                .on_press(Message::RequestEnter(current_path_dir.clone()))
                .into()];
            newbtns.append(&mut dirbtn);
            dirbtn = newbtns;
        }

        rowvec.append(&mut dirbtn);
        rowvec.append(&mut vec![
            checkbox("show hide", show_hide, Message::RequestShowHide)
                .size(20)
                .into(),
            checkbox("preivew image", preview_image, Message::RequestShowImage)
                .size(20)
                .into(),
        ]);
        container(
            row(rowvec)
                .spacing(10)
                .padding(5)
                .align_items(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .into()
    }

    pub fn enter(dir: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let (count, iter) = ls_dir_pre(dir)?;
        let mut enterdir = Self {
            is_end: false,
            iter,
            infos: Vec::new(),
            current_dir: dir.to_path_buf(),
        };
        if count < 1000 {
            while !enterdir.is_end {
                let _ = enterdir.polldir();
            }
        }
        Ok(enterdir)
    }

    pub fn ls_end(&self) -> bool {
        self.is_end
    }

    pub fn polldir(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(file) = self.iter.next() else {
            self.is_end = true;
            return Ok(());
        };
        let name = file
            .file_name()
            .into_string()
            .map_err(|f| format!("Invalid entry: {:?}", f))?;
        let metadata = file.metadata()?;
        let path = file.path();
        use std::os::unix::fs::MetadataExt;
        let permission = parse_permission(metadata.mode());
        let mime = &MIME;
        if metadata.is_symlink() {
            let realpath = fs::read_link(&path).unwrap();
            if path.is_dir() {
                self.infos.push(FsInfo::Dir {
                    path,
                    name,
                    permission,
                    symlink: Some(realpath),
                });
            } else {
                let mimeinfo = mime.get_mime_types_from_file_name(&name);
                let icon = mimeinfo
                    .first()
                    .and_then(|info| mime.lookup_generic_icon_name(info))
                    .unwrap_or(TEXT_ICON.to_string());
                self.infos.push(FsInfo::File {
                    path,
                    icon,
                    permission,
                    name,
                    symlink: Some(realpath),
                    mimeinfo,
                });
            }
            return Ok(());
        }
        if metadata.is_dir() {
            self.infos.push(FsInfo::Dir {
                path,
                name,
                permission,
                symlink: None,
            });
        } else {
            let mimeinfo = mime.get_mime_types_from_file_name(&name);
            let icon = mimeinfo
                .first()
                .and_then(|info| mime.lookup_generic_icon_name(info))
                .unwrap_or(TEXT_ICON.to_string());
            self.infos.push(FsInfo::File {
                path,
                icon,
                permission,
                name,
                symlink: None,
                mimeinfo,
            })
        }
        self.infos.sort_by(|a, b| {
            a.name()
                .to_string()
                .partial_cmp(&b.name().to_string())
                .unwrap()
        });
        Ok(())
    }

    fn fs_infos(&self) -> &Vec<FsInfo> {
        &self.infos
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
        mimeinfo: Vec<Mime>,
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

fn ls_dir_pre(dir: &PathBuf) -> Result<(usize, std::iter::Flatten<ReadDir>), Box<dyn Error>> {
    if !dir.is_dir() {
        return Err("Dir is not file".into());
    }
    let count = fs::read_dir(dir)?.count();
    Ok((count, fs::read_dir(dir)?.flatten()))
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
        if self.is_file() {
            return true;
        }
        self.is_dir() && self.path().read_dir().is_ok()
    }

    pub fn is_svg(&self) -> bool {
        let FsInfo::File {
            path,
            icon,
            permission,
            name,
            symlink,
            mimeinfo,
        } = self
        else {
            return false;
        };
        mimeinfo.contains(&Mime::from_str("image/svg+xml").unwrap())
    }

    pub fn is_image(&self) -> bool {
        self.icon() == "image-x-generic"
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

    fn get_icon_handle(&self) -> svg::Handle {
        if let Some(icon) = get_icon("Adwaita", self.icon()) {
            return svg::Handle::from_path(icon);
        }
        if self.is_dir() {
            svg::Handle::from_memory(DIR_IMAGE)
        } else {
            svg::Handle::from_memory(TEXT_IMAGE)
        }
    }

    fn get_icon(&self, preview_image: bool) -> Element<Message> {
        if self.is_svg() {
            return svg(svg::Handle::from_path(self.path())).into();
        }
        if self.is_image() && preview_image {
            return image(self.path()).into();
        }
        svg(self.get_icon_handle()).into()
    }

    fn view(&self, select_dir: bool, preview_image: bool) -> Element<Message> {
        let mut file_btn = button(self.get_icon(preview_image))
            .padding(10)
            .width(BUTTON_WIDTH)
            .height(BUTTON_WIDTH);

        let dir_can_enter = self.is_dir() && self.is_readable();

        let can_selected = self.is_readable() && (self.is_dir() == select_dir);
        if dir_can_enter || can_selected {
            file_btn = file_btn.style(theme::Button::Secondary);
        }

        if dir_can_enter {
            file_btn = file_btn.on_press(Message::RequestEnter(self.path()));
        }

        let bottom_text: Element<Message> = if can_selected {
            file_btn = file_btn.on_press(Message::RequestSelect);
            container(checkbox(self.name(), false, |_| Message::Check).width(BUTTON_WIDTH))
                .width(Length::Fill)
                .into()
        } else {
            container(
                text(self.name())
                    .width(BUTTON_WIDTH)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .into()
        };

        let tocontainer = column![file_btn, bottom_text];
        container(tocontainer)
            .height(COLUMN_WIDTH)
            .width(Length::Fill)
            .center_x()
            .into()
    }
}
