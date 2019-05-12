extern crate termion;

use std::collections::HashMap;

use std::io::{self, BufRead};
use termion::{color, style};

fn main() {
    // keeps track of how often each line has occurred
    let mut line_counts = HashMap::new();

    let mut lines = LineCollector::new();
    let mut has_cleared_screen = false;

    for line_or_error in io::stdin().lock().lines() {
        let line = match line_or_error {
            Ok(line) => line.trim().to_string(), // input line is utf8, so trim it
            Err(_) => continue, // input is non-utf8 (maybe binary), ignore the error for now
        };

        lines.insert(line.clone());

        let line_count = line_counts.entry(line).or_insert(0u32);
        *line_count += 1;

        // get the keys and order them by count and then the actual string
        let mut keys: Vec<_> = line_counts.keys().collect();
        keys.sort_unstable_by_key(|k| (line_counts[*k], *k));

        // TODO don't rewrite if nothing has changed since last time (on the screen)

        // display current status
        let (twidth, theight) = termion::terminal_size().unwrap();

        // clear the screen the first time, subsequently we make sure we call clear::UntilNewLine for each line
        // this reduces the flicker since each character is modified only once per loop
        if !has_cleared_screen {
            has_cleared_screen = true;
            print!("{}", termion::clear::All);
        }

        print!(
            "{}{}Lines total: {}, Unique lines: {}{}",
            termion::cursor::Goto(1, 1),
            color::Fg(color::Yellow),
            lines.num_total(),
            lines.num_unique(),
            style::Reset
        );

        // iterate over references in reverse to display top first
        for (i, key) in keys.iter().rev().enumerate() {
            if i + 1 >= theight as usize {
                break;
            }

            // render the full output line
            let out_line = format!("{:width$}: {}", line_counts[*key], key, width = 5);

            let mut out_chars: Vec<char> = out_line.chars().collect();
            out_chars.truncate(twidth as usize);

            let out_line: String = out_chars.into_iter().collect();

            // clip the line to terminal width
            print!("\n{}{}", out_line, termion::clear::UntilNewline);
        }
    }
}

#[derive(Default, Debug)]
pub struct LineCollector {
    line_counts: HashMap<String, usize>,
    total_lines: usize,
}

impl LineCollector {
    pub fn new() -> Self {
        LineCollector {
            ..Default::default()
        }
    }

    pub fn num_total(&self) -> usize {
        self.total_lines
    }

    pub fn num_unique(&self) -> usize {
        self.line_counts.len()
    }

    pub fn insert(&mut self, line: String) {
        self.total_lines += 1;

        let line_count = self.line_counts.entry(line).or_insert(0);
        *line_count += 1;
    }

    pub fn iter(&self) -> LineCollectorResultIter {
        LineCollectorResultIter::new(&self.line_counts)
    }
}

pub struct LineCollectorResultIter<'a> {
    idx: usize,
    sorted_lines: Vec<&'a String>, // TODO use str
}

// TODO implemenmt IntoIterator?!
impl<'a> LineCollectorResultIter<'a> {
    fn new(line_counts: &'a HashMap<String, usize>) -> Self {
        let mut sorted_lines: Vec<_> = line_counts.keys().collect();
        sorted_lines.sort_unstable_by_key(|k| (line_counts[*k], *k));
        LineCollectorResultIter {
            idx: 0,
            sorted_lines,
        }
    }
}

impl<'a> Iterator for LineCollectorResultIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        // TODO use iterator, not index. Maybe even stor ethe iterator instead of vec?
        if self.idx >= self.sorted_lines.len() {
            None
        } else {
            self.idx += 1;
            Some(self.sorted_lines[self.idx - 1])
        }
    }
}
