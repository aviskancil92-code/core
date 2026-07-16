use std::path::{Path, PathBuf};

pub fn resolve_path(p: &str, parent_file: Option<&PathBuf>) -> Option<PathBuf> {
    let path = Path::new(p);

    if path.is_absolute() {
        // return Some(p.to_string());
        return normalize_file_name(path);
    }

    match parent_file {
        Some(x) => {
            // todo:
            // If the file path is relative, the compiler scans directories in the following order to find the file:
            // 1. directory of the file with the directive
            // 2. data folder for the current edit mode
            // 3. Sanny Builder root directory
            // 4. the game directory
            let dir_name = Path::new(&x).parent()?;
            let abs_name = dir_name.join(path);

            // Some(String::from(abs_name.to_str()?))
            normalize_file_name(&abs_name)
        }
        None => None,
    }
}



pub fn normalize_file_name(file_name: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use normpath::PathExt;
        Some(file_name.normalize_virtually().ok()?.into_path_buf())
    }

    // `normpath`'s `normalize_virtually` (lexical normalization, no filesystem
    // access) is only available on Windows. On other targets -- including
    // wasm32, which has no real filesystem to canonicalize against anyway --
    // fall back to a plain lexical normalization (collapse `.`/`..`
    // components without touching disk).
    #[cfg(not(windows))]
    {
        use std::path::Component;
        let mut result = PathBuf::new();
        for component in file_name.components() {
            match component {
                Component::ParentDir => {
                    result.pop();
                }
                Component::CurDir => {}
                other => result.push(other.as_os_str()),
            }
        }
        Some(result)
    }
}
