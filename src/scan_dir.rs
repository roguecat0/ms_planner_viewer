use std::fs;
use std::io;
use std::path::Path;

pub fn move_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<bool> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.exists() {
        return Ok(false);
    }

    match fs::rename(src, dst) {
        Ok(_) => return Ok(true),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }

            // copy and preserve permissions, then remove original
            match fs::copy(src, dst) {
                Ok(_) => {
                    let metadata = fs::metadata(src)?;
                    fs::set_permissions(dst, metadata.permissions())?;
                    fs::remove_file(src)?;
                    Ok(true)
                }
                Err(copy_err) => {
                    // try to clean up partial destination if it exists
                    let _ = fs::remove_file(dst);
                    Err(copy_err)
                }
            }
        }
        Err(e) => Err(e),
    }
}
