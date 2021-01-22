use filetime::FileTime;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn read_to_string(path: &Path) -> io::Result<String> {
    let mut content = std::fs::read_to_string(path)?;

    // strip break line at file end
    if content.ends_with('\n') {
        content.truncate(content.len() - 1);
        if content.ends_with('\r') {
            content.truncate(content.len() - 1);
        }
    }

    Ok(content)
}

fn find_rustfmt() -> io::Result<Option<PathBuf>> {
    let mut toolchain_dir = home::rustup_home()?;
    toolchain_dir.push("toolchains");
    for e in fs::read_dir(toolchain_dir)? {
        let mut path = e?.path();
        path.push("bin");
        path.push("rustfmt");
        if path.exists() {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Format block expression using `rustfmt` command
pub fn rustfmt_block(source: &str) -> io::Result<String> {
    let rustfmt = match find_rustfmt()? {
        Some(p) => p,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "rustfmt command not found",
            ))
        }
    };

    let mut new_source = String::with_capacity(source.len() + 11);
    new_source.push_str("fn render()");
    new_source.push_str(source);

    let mut child = Command::new(rustfmt)
        .args(&["--emit", "stdout", "--color", "never", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| io::Error::from(io::ErrorKind::BrokenPipe))?;
    stdin.write_all(new_source.as_bytes())?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        let mut s =
            String::from_utf8(output.stdout).expect("rustfmt output is non-UTF-8!");
        let brace_offset = s.find('{').unwrap();
        s.replace_range(..brace_offset, "");
        Ok(s)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "rustfmt command failed",
        ))
    }
}

pub fn copy_filetimes(input: &Path, output: &Path) -> io::Result<()> {
    let mtime = fs::metadata(input)
        .and_then(|metadata| metadata.modified())
        .map_or(FileTime::zero(), |time| FileTime::from_system_time(time));

    filetime::set_file_times(output, mtime, mtime)
}
