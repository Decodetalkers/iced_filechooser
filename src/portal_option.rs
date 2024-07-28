use std::{
    ffi::{CString, OsStr},
    os::unix::ffi::OsStrExt,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct FilePath(CString);

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        OsStr::from_bytes(self.0.as_bytes()).as_ref()
    }
}

impl Serialize for FilePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes_with_nul())
    }
}

impl<'de> Deserialize<'de> for FilePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        let c_string = CString::from_vec_with_nul(bytes)
            .map_err(|_| serde::de::Error::custom("Bytes are not nul-terminated"))?;

        Ok(Self(c_string))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum FilterType {
    GlobPattern = 0,
    MimeType = 1,
}

impl FilterType {
    /// Whether it is a mime type filter.
    fn is_mimetype(&self) -> bool {
        matches!(self, FilterType::MimeType)
    }

    /// Whether it is a glob pattern type filter.
    fn is_pattern(&self) -> bool {
        matches!(self, FilterType::GlobPattern)
    }
}

impl FileFilter {
    /// Create a new file filter
    ///
    /// # Arguments
    ///
    /// * `label` - user-visible name of the file filter.
    pub fn new(label: &str) -> Self {
        Self(label.to_owned(), vec![])
    }

    /// Adds a mime type to the file filter.
    #[must_use]
    pub fn mimetype(mut self, mimetype: &str) -> Self {
        self.1.push((FilterType::MimeType, mimetype.to_owned()));
        self
    }

    /// Adds a glob pattern to the file filter.
    #[must_use]
    pub fn glob(mut self, pattern: &str) -> Self {
        self.1.push((FilterType::GlobPattern, pattern.to_owned()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct FileFilter(String, Vec<(FilterType, String)>);

impl FileFilter {
    /// The label of the filter.
    pub fn label(&self) -> &str {
        &self.0
    }

    /// List of mimetypes filters.
    pub fn mimetype_filters(&self) -> Vec<&str> {
        self.1
            .iter()
            .filter_map(|(type_, string)| type_.is_mimetype().then_some(string.as_str()))
            .collect()
    }

    /// List of glob patterns filters.
    pub fn pattern_filters(&self) -> Vec<&str> {
        self.1
            .iter()
            .filter_map(|(type_, string)| type_.is_pattern().then_some(string.as_str()))
            .collect()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// Presents the user with a choice to select from or as a checkbox.
pub struct Choice(String, String, Vec<(String, String)>, String);

impl Choice {
    /// Creates a checkbox choice.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier of the choice.
    /// * `label` - user-visible name of the choice.
    /// * `state` - the initial state value.
    pub fn boolean(id: &str, label: &str, state: bool) -> Self {
        Self::new(id, label, &state.to_string())
    }

    /// Creates a new choice.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier of the choice.
    /// * `label` - user-visible name of the choice.
    /// * `initial_selection` - the initially selected value.
    pub fn new(id: &str, label: &str, initial_selection: &str) -> Self {
        Self(
            id.to_owned(),
            label.to_owned(),
            vec![],
            initial_selection.to_owned(),
        )
    }

    /// Adds a (key, value) as a choice.
    #[must_use]
    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.2.push((key.to_owned(), value.to_owned()));
        self
    }

    /// The choice's unique id
    pub fn id(&self) -> &str {
        &self.0
    }

    /// The user visible label of the choice.
    pub fn label(&self) -> &str {
        &self.1
    }

    /// Pairs of choices.
    pub fn pairs(&self) -> Vec<(&str, &str)> {
        self.2
            .iter()
            .map(|(x, y)| (x.as_str(), y.as_str()))
            .collect::<Vec<_>>()
    }

    /// The initially selected value.
    pub fn initial_selection(&self) -> &str {
        &self.3
    }
}

#[derive(Debug, Clone)]
pub enum FileChoosen {
    OpenFile {
        handle_token: String,
        accept_label: String,
        modal: bool,
        multiple: bool,
        directory: bool,
        filters: Vec<FileFilter>,
        current_filter: Option<FileFilter>,
        choices: Vec<Choice>,
        current_folder: Option<FilePath>,
    },
    SaveFile {
        handle_token: String,
        accept_label: String,
        modal: bool,
        filters: Vec<FileFilter>,
        current_filter: Option<FileFilter>,
        choices: Vec<Choice>,
        current_folder: FilePath,
        current_file: FilePath,
    },
}

impl Default for FileChoosen {
    fn default() -> Self {
        Self::OpenFile {
            handle_token: "".to_string(),
            accept_label: "".to_string(),
            modal: true,
            multiple: false,
            directory: false,
            filters: Vec::new(),
            current_filter: None,
            choices: Vec::new(),
            current_folder: None,
        }
    }
}

impl FileChoosen {
    pub fn is_filechooser(&self) -> bool {
        matches!(self, FileChoosen::OpenFile { .. })
    }

    pub fn is_muti_filechooser(&self) -> bool {
        matches!(self, FileChoosen::OpenFile { multiple: true, .. })
    }

    pub fn is_savefile(&self) -> bool {
        !self.is_filechooser()
    }

    pub fn is_directory(&self) -> bool {
        matches!(
            self,
            FileChoosen::OpenFile {
                directory: true,
                ..
            }
        )
    }

    pub fn filters(&self) -> &[FileFilter] {
        match self {
            Self::OpenFile { filters, .. } => filters,
            Self::SaveFile { filters, .. } => filters,
        }
    }

    pub fn choices(&self) -> &[Choice] {
        match self {
            Self::OpenFile { choices, .. } => choices,
            Self::SaveFile { choices, .. } => choices,
        }
    }

    pub fn handle_token(&self) -> &str {
        match self {
            Self::OpenFile { handle_token, .. } => handle_token,
            Self::SaveFile { handle_token, .. } => handle_token,
        }
    }

    pub fn accept_label(&self) -> &str {
        match self {
            Self::OpenFile { accept_label, .. } => accept_label,
            Self::SaveFile { accept_label, .. } => accept_label,
        }
    }

    pub fn is_modal(&self) -> bool {
        match self {
            Self::OpenFile { modal, .. } => *modal,
            Self::SaveFile { modal, .. } => *modal,
        }
    }
}
