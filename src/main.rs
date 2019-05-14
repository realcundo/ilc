extern crate termion;

use std::io::{self, BufRead};
use termion::{color, style};

mod linecollector;
use linecollector::LineCollector;

fn main() {
    // keeps track of how often each line has occurred
    let mut lines = LineCollector::new();
    let mut has_cleared_screen = false;

    for line_or_error in io::stdin().lock().lines() {
        let line = match line_or_error {
            Ok(line) => line.trim().to_string(), // input line is utf8, so trim it
            Err(_) => continue, // input is non-utf8 (maybe binary), ignore the error for now
        };

        // create a clone of the string that is owned by "lines"
        lines.insert(line.clone());

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
        for (i, key) in lines.iter().enumerate() {
            if i + 1 >= theight as usize {
                break;
            }

            // render the full output line
            let out_line = format!("{:width$}: {}", lines[key], key, width = 5);

            let mut out_chars: Vec<char> = out_line.chars().collect();
            out_chars.truncate(twidth as usize);

            let out_line: String = out_chars.into_iter().collect();

            // clip the line to terminal width
            print!("\n{}{}", out_line, termion::clear::UntilNewline);
        }
    }
}
