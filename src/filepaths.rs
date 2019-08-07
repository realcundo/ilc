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

impl FilePathParser {
    pub fn new(files: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let dash = PathBuf::from("-");

        // assume the number of "-" in the input is at most one so it's more
        // efficient to simply copy the input sequence and then remove "-".
        let mut files = files.into_iter().map(Into::into).collect::<Vec<_>>();
        let old_size = files.len();

        files.retain(|f| *f != dash);

        // stdin should be true if we've removed at least one dash or if the input
        // is empty
        let has_stdin = (files.len() != old_size) || (old_size == 0);

        Self { files, has_stdin }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_stdin_by_default() {
        let input: Vec<PathBuf> = vec![];
        let expected = input.clone();

        let fp = FilePathParser::new(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn single_file() {
        let input = vec![PathBuf::from("file1")];
        let expected = input.clone();

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, false);
    }

    #[test]
    fn stdin_only() {
        let input = vec![PathBuf::from("-")];

        let expected: Vec<PathBuf> = vec![];

        let fp = FilePathParser::new(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }

    #[test]
    fn stdin_multiple_times() {
        let input = vec![PathBuf::from("-"), PathBuf::from("-"), PathBuf::from("-")];

        let expected: Vec<PathBuf> = vec![];

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

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

        let fp = FilePathParser::new(input);

        assert_eq!(fp.files, expected);
        assert_eq!(fp.has_stdin, true);
    }
}
