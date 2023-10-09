use iced::widget::{button, checkbox, column, container, image, row, scrollable, svg, text, Space};
use iced::{alignment, Font};
use iced::{theme, Element, Length};
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};

use iced_aw::{split, Grid, Split};

use crate::utils::get_icon;

use mime::Mime;
use xdg_mime::SharedMimeInfo;

use once_cell::sync::Lazy;

static MIME: Lazy<SharedMimeInfo> = Lazy::new(SharedMimeInfo::new);

const TEXT_IMAGE: &[u8] = include_bytes!("../resources/text-plain.svg");

const DIR_IMAGE: &[u8] = include_bytes!("../resources/inode-directory.svg");

const GO_PREVIOUS: &[u8] = include_bytes!("../resources/go-previous.svg");

const LOADING: &[u8] = include_bytes!("../resources/Loading_icon_no_fade.svg");

const DIR_ICON: &str = "inode-directory";
const TEXT_ICON: &str = "text-plain";

use crate::Message;

const COLUMN_WIDTH: f32 = 200.0;

const BUTTON_WIDTH: f32 = 170.0;

#[derive(Debug)]
pub struct DirUnit {
    is_end: bool,
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

    pub fn append_infos(&mut self, mut dirs: Vec<FsInfo>) {
        self.infos.append(&mut dirs);
        self.infos.sort_by(|a, b| {
            a.name()
                .to_string()
                .partial_cmp(&b.name().to_string())
                .unwrap()
        });
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

    fn find_unit(&self, path: &Path) -> Option<&FsInfo> {
        self.fs_infos().iter().find(|iter| 'selected: {
            let Ok(origin_path) = path.canonicalize() else {
                break 'selected false;
            };
            let Ok(self_path) = iter.path().canonicalize() else {
                break 'selected false;
            };
            self_path.as_os_str() == origin_path.as_os_str()
        })
    }

    fn main_grid(
        &self,
        show_hide: bool,
        preview_image: bool,
        right_spliter: &Option<u16>,
        current_selected: &Option<PathBuf>,
        select_dir: bool,
    ) -> Element<Message> {
        let mut grid = Grid::with_column_width(COLUMN_WIDTH);
        let filter_way = |dir: &&FsInfo| show_hide || !dir.is_hidden();
        let infowidth = self.fs_infos().iter().filter(filter_way).count();
        if infowidth > 500 {
            let mut iter = self.fs_infos().iter().filter(filter_way);
            for _ in 0..500 {
                let dir = iter.next().unwrap();
                grid = grid.push(dir.view(select_dir, preview_image, current_selected));
            }
        } else {
            for dir in self.fs_infos().iter().filter(filter_way) {
                grid = grid.push(dir.view(select_dir, preview_image, current_selected));
            }
        };
        let rightviewinfo = current_selected.as_ref().and_then(|p| self.find_unit(p));
        match rightviewinfo {
            Some(info) => Split::new(
                scrollable(container(grid).center_x().width(Length::Fill)),
                info.right_view(),
                *right_spliter,
                split::Axis::Vertical,
                Message::RequestAdjustRightSpliter,
            )
            .width(Length::Fill)
            .padding(10.0)
            .into(),
            None => scrollable(container(grid).center_x().width(Length::Fill)).into(),
        }
    }

    fn loading_page(&self) -> Element<Message> {
        container(column![
            Space::new(Length::Fill, Length::Fill),
            row![
                Space::new(Length::Fill, 20),
                text("please waiting").font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
                Space::new(5, 20),
                svg(svg::Handle::from_memory(LOADING)).width(20).height(20)
            ]
            .align_items(iced::Alignment::End)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn bottom_view(
        &self,
        show_hide: bool,
        preview_image: bool,
        right_spliter: &Option<u16>,
        current_selected: &Option<PathBuf>,
        select_dir: bool,
    ) -> Element<Message> {
        if self.is_end {
            self.main_grid(
                show_hide,
                preview_image,
                right_spliter,
                current_selected,
                select_dir,
            )
        } else {
            self.loading_page()
        }
    }

    pub fn view(
        &self,
        show_hide: bool,
        preview_image: bool,
        right_spliter: &Option<u16>,
        current_selected: &Option<PathBuf>,
        select_dir: bool,
    ) -> Element<Message> {
        column![
            self.title_bar(show_hide, preview_image),
            self.bottom_view(
                show_hide,
                preview_image,
                right_spliter,
                current_selected,
                select_dir
            )
        ]
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

    pub fn enter(dir: &Path) -> Self {
        Self {
            is_end: false,
            infos: Vec::new(),
            current_dir: dir.to_path_buf(),
        }
    }

    pub fn set_end(&mut self) {
        self.is_end = true;
    }

    fn fs_infos(&self) -> &Vec<FsInfo> {
        &self.infos
    }
}
pub async fn update_dir_infos<P: AsRef<Path>>(path: P) -> Vec<FsInfo> {
    let mut fs_infos = Vec::new();
    let Ok(dirs) = fs::read_dir(path) else {
        return fs_infos;
    };
    for file in dirs.flatten() {
        let Ok(name) = file.file_name().into_string() else {
            continue;
        };
        let Ok(metadata) = file.metadata() else {
            continue;
        };

        tokio::time::sleep(std::time::Duration::from_nanos(5)).await;

        let path = file.path();
        use std::os::unix::fs::MetadataExt;
        let permission = parse_permissions(metadata.mode());
        let mime = &MIME;
        if metadata.is_symlink() {
            let realpath = tokio::fs::read_link(&path).await.unwrap();
            if path.is_dir() {
                fs_infos.push(FsInfo::Dir {
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
                fs_infos.push(FsInfo::File {
                    path,
                    icon,
                    permission,
                    name,
                    symlink: Some(realpath),
                    mimeinfo,
                });
            }
            continue;
        }
        if metadata.is_dir() {
            fs_infos.push(FsInfo::Dir {
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
            fs_infos.push(FsInfo::File {
                path,
                icon,
                permission,
                name,
                symlink: None,
                mimeinfo,
            })
        }
    }

    fs_infos
}

#[derive(Debug, Clone)]
pub enum FsInfo {
    File {
        path: PathBuf,
        icon: String,
        permission: String,
        name: String,
        symlink: Option<PathBuf>,
        mimeinfo: Vec<Mime>,
    },
    Dir {
        path: PathBuf,
        name: String,
        permission: String,
        symlink: Option<PathBuf>,
    },
}
fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
    .to_string()
}

fn parse_permissions(mode: u32) -> String {
    let user = triplet(mode, S_IRUSR, S_IWUSR, S_IXUSR);
    let group = triplet(mode, S_IRGRP, S_IWGRP, S_IXGRP);
    let other = triplet(mode, S_IROTH, S_IWOTH, S_IXOTH);
    [user, group, other].join("")
}

#[allow(unused)]
impl FsInfo {
    pub fn permission(&self) -> &str {
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

    fn get_text_icon(&self, theme: &str) -> Option<String> {
        let icon = self.icon();
        if icon != "text-x-generic" {
            return None;
        }

        let FsInfo::File { mimeinfo, .. } = self else {
            return None;
        };

        let Some(iconname) = mimeinfo.first() else {
            return None;
        };

        let newicon = iconname.to_string().replace('/', "-");
        get_icon(theme, newicon.as_str())
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

    fn get_default_generate_icon_handle(&self, theme: &str) -> svg::Handle {
        if let Some(icon) = get_icon(theme, self.icon()) {
            return svg::Handle::from_path(icon);
        }
        if self.is_dir() {
            svg::Handle::from_memory(DIR_IMAGE)
        } else {
            svg::Handle::from_memory(TEXT_IMAGE)
        }
    }

    fn get_default_icon_handle(&self) -> svg::Handle {
        if let Some(icon) = self.get_text_icon("Adwaita") {
            return svg::Handle::from_path(icon);
        }
        self.get_default_generate_icon_handle("Adwaita")
    }

    fn get_icon(&self, preview_image: bool) -> Element<Message> {
        if self.is_svg() {
            return svg(svg::Handle::from_path(self.path()))
                .height(100)
                .width(Length::Fill)
                .into();
        }
        if self.is_image() && preview_image {
            return image(self.path()).width(Length::Fill).into();
        }
        svg(self.get_default_icon_handle())
            .height(100)
            .width(Length::Fill)
            .into()
    }

    fn right_view(&self) -> Element<Message> {
        column![
            self.get_icon(true),
            text(self.permission())
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill),
            text(self.name())
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill)
        ]
        .into()
    }

    fn view(
        &self,
        select_dir: bool,
        preview_image: bool,
        current_selected: &Option<PathBuf>,
    ) -> Element<Message> {
        let mut file_btn = button(self.get_icon(preview_image))
            .padding(10)
            .width(BUTTON_WIDTH)
            .height(BUTTON_WIDTH);

        let is_selected = current_selected.as_ref().is_some_and(|path| 'selected: {
            let Ok(origin_path) = path.canonicalize() else {
                break 'selected false;
            };
            let Ok(self_path) = self.path().canonicalize() else {
                break 'selected false;
            };
            origin_path.as_os_str() == self_path.as_os_str()
        });

        let dir_can_enter = self.is_dir() && self.is_readable();

        let can_selected = self.is_readable() && (self.is_dir() == select_dir);
        if dir_can_enter || can_selected {
            file_btn = file_btn.style(theme::Button::Secondary);
        }

        if dir_can_enter {
            file_btn = file_btn.on_press(Message::RequestEnter(self.path()));
        }

        let bottom_text: Element<Message> = if can_selected {
            file_btn = file_btn.on_press(Message::RequestSelect(self.path().clone()));
            if is_selected {
                file_btn = file_btn.style(theme::Button::Primary);
            }
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
