use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

fn read_ignoring_utf_errors(path: &PathBuf) -> String {
    let mut f = File::open(path).unwrap_or_else(|_| panic!("McFly error: {:?} file not found", &path));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap_or_else(|_| panic!("McFly error: Unable to read from {:?}", &path));
    String::from_utf8_lossy(&buffer).to_string()
}

#[allow(clippy::if_same_then_else)]
fn has_leading_timestamp(line: &str) -> bool {
    let mut matched_chars = 0;

    for (index, c) in line.chars().enumerate() {
        if index == 0 && c == '#' {
            matched_chars += 1;
        } else if index > 0 && index < 11 && (c.is_digit(10))  {
            matched_chars += 1;
        } else if index > 11 {
            break;
        }
    }

    matched_chars == 11
}

pub fn bash_history_file_path() -> PathBuf {
    let path =
        PathBuf::from(env::var("HISTFILE").unwrap_or_else(|err| panic!(format!("McFly error: Please ensure HISTFILE is set for your shell ({})", err))));
    fs::canonicalize(&path).unwrap_or_else(|err| panic!(format!("McFly error: The contents of $HISTFILE appear invalid ({})", err)))
}

pub fn full_history(path: &PathBuf) -> Vec<String> {
    let bash_history_contents = read_ignoring_utf_errors(&path);

    bash_history_contents
        .split('\n')
        .filter(|line| !has_leading_timestamp(line) && !line.is_empty())
        .map(String::from)
        .collect::<Vec<String>>()
}

pub fn last_history_line(path: &PathBuf) -> Option<String> {
    // Could switch to https://github.com/mikeycgto/rev_lines
    full_history(path).last().map(|s| s.trim().to_string())
}

pub fn delete_last_history_entry_if_search(path: &PathBuf) {
    let bash_history_contents = read_ignoring_utf_errors(&path);

    let mut lines = bash_history_contents
        .split('\n')
        .map(String::from)
        .collect::<Vec<String>>();

    if !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }

    if lines.is_empty() || !lines[lines.len() - 1].starts_with("#mcfly:") {
        return; // Abort if empty or the last line isn't a comment.
    }

    lines.pop();

    if !lines.is_empty() && has_leading_timestamp(&lines[lines.len() - 1]) {
        lines.pop();
    }

    lines.push(String::from("")); // New line at end of file expected by bash.

    fs::write(&path, lines.join("\n")).unwrap_or_else(|_| panic!("McFly error: Unable to update {:?}", &path));
}

pub fn delete_lines(path: &PathBuf, command: &str) {
    let history_contents = read_ignoring_utf_errors(&path);

    let lines = history_contents
        .split('\n')
        .map(String::from)
        .filter(|cmd| !cmd.eq(command))
        .collect::<Vec<String>>();

    fs::write(&path, lines.join("\n")).unwrap_or_else(|_| panic!("McFly error: Unable to update {:?}", &path));
}

pub fn append_history_entry(command: &str, path: &PathBuf) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|err| panic!(format!("McFly error: please make sure HISTFILE exists ({})", err)));

    if let Err(e) = writeln!(file, "{}", command) {
        eprintln!("Couldn't append to file {:?}: {}", &path, e);
    }
}

#[cfg(test)]
mod tests {
    use super::has_leading_timestamp;

    #[test]
    fn has_leading_timestamp_works() {
        assert_eq!(false, has_leading_timestamp("abc"));
        assert_eq!(false, has_leading_timestamp("#abc"));
        assert_eq!(false, has_leading_timestamp("#123456"));
        assert_eq!(true, has_leading_timestamp("#1234567890"));
        assert_eq!(false, has_leading_timestamp("#123456789"));
        assert_eq!(false, has_leading_timestamp("# 1234567890"));
        assert_eq!(false, has_leading_timestamp("1234567890"));
        assert_eq!(false, has_leading_timestamp("hello 1234567890"));
    }
}
