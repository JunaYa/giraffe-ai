use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use sha1::{Digest, Sha1};

use crate::{AppError, ChatFile};

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        let ext = filename.split('.').next_back().unwrap_or("");
        Self {
            ws_id,
            ext: ext.to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // split hash into 3 parts, first 2 with 3
    pub fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!(
            "{ws_id}/{part1}/{part2}/{part3}.{ext}",
            ws_id = self.ws_id,
            ext = self.ext
        )
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(format!(
                "Invalid chat file path: {s}"
            )));
        };

        let parts = s.split('/').collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(format!(
                "File path {s} does not valid"
            )));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id {}",
                parts[0]
            )));
        };

        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name {}",
                parts[3]
            )));
        };

        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
