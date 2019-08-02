use std::{convert::From, path::PathBuf};

/// Parser that reads `FileBuf` vector and collects it internally, detecting "-"
/// filenames and if one of more is found `has_stdin` is set to `true`. If input
/// vector is empty `has_stdin` is set to `true`.
/// TODO ["--", "-"] should be interpreted as filename "-" instead of stdin
#[derive(Debug)]
pub struct FilePathParser {
    pub files: Vec<PathBuf>,
    pub has_stdin: bool,
}

impl From<Vec<PathBuf>> for FilePathParser {
    fn from(files: Vec<PathBuf>) -> Self {
        let mut result = FilePathParser {
            // we'll have at most files.len() elements
            files: Vec::with_capacity(files.len()),
            has_stdin: false,
        };

        let dash = PathBuf::from("-");

        // copy all elements except dash. Dash means stdin.
        for f in files {
            if f == dash {
                result.has_stdin = true;
            } else {
                result.files.push(f);
            }
        }

        // if no files, default to stdin
        if result.files.is_empty() {
            result.has_stdin = true
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_stdin_by_default() {
        let input = vec![];
        let expected = input.clone();

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn single_file() {
        let input = vec![PathBuf::from("file1")];
        let expected = input.clone();

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, false);
    }

    #[test]
    fn multiple_filenames() {
        let input = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file3"),
        ];
        let expected = input.clone();

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, false);
    }

    #[test]
    fn duplicate_filenames() {
        let input = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file1"),
        ];
        let expected = input.clone();

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, false);
    }

    #[test]
    fn stdin_only() {
        let input = vec![PathBuf::from("-")];

        let expected: Vec<PathBuf> = vec![];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn stdin_multiple_times() {
        let input = vec![PathBuf::from("-"), PathBuf::from("-"), PathBuf::from("-")];

        let expected: Vec<PathBuf> = vec![];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn stdin_before_filenames() {
        let input = vec![
            PathBuf::from("-"),
            PathBuf::from("file1"),
            PathBuf::from("file2"),
        ];

        let expected = vec![PathBuf::from("file1"), PathBuf::from("file2")];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn stdin_after_filenames() {
        let input = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("-"),
        ];

        let expected = vec![PathBuf::from("file1"), PathBuf::from("file2")];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn stdin_between_filenames() {
        let input = vec![
            PathBuf::from("file1"),
            PathBuf::from("-"),
            PathBuf::from("file2"),
        ];

        let expected = vec![PathBuf::from("file1"), PathBuf::from("file2")];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn mixed_stdin_and_duplicate_filenames() {
        let input = vec![
            PathBuf::from("-"),
            PathBuf::from("file1"),
            PathBuf::from("-"),
            PathBuf::from("file2"),
            PathBuf::from("-"),
            PathBuf::from("file1"),
            PathBuf::from("file2"),
        ];

        let expected = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file1"),
            PathBuf::from("file2"),
        ];

        let fp = FilePathParser::from(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }
}
