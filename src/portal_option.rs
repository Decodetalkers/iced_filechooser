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

#[derive(Debug, Clone)]
pub struct Filter(String, Vec<(FilterType, String)>);

#[derive(Clone, Serialize, Deserialize, Debug)]
/// Presents the user with a choice to select from or as a checkbox.
pub struct Choice(String, String, Vec<(String, String)>, String);

#[derive(Debug, Clone)]
pub enum FileChoosen {
    OpenFile {
        handle_token: String,
        accept_label: String,
        modal: bool,
        multiple: bool,
        directory: bool,
        filters: Vec<Filter>,
        current_filter: Option<Filter>,
        choices: Vec<Choice>,
        current_folder: FilePath,
    },
    SaveFile {
        handle_token: String,
        accept_label: String,
        modal: bool,
        filters: Vec<Filter>,
        current_filter: Option<Filter>,
        choices: Vec<Choice>,
        current_folder: FilePath,
        current_file: FilePath,
    },
}
