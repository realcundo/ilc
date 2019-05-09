extern crate termion;

use std::collections::HashMap;

use std::io::{self, BufRead};
use termion::{color, style};


fn main() {
    // keeps track of how often each line has occurred
    let mut line_counts = HashMap::new();

    let mut lines_total = 0u32;

    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let line = line.trim().to_string();
        lines_total += 1;

        let line_count = line_counts.entry(line).or_insert(0u32);
        *line_count += 1;

        // get the keys and order them by count and then the actual string
        let mut keys: Vec<_> = line_counts.keys().collect();
        keys.sort_unstable_by_key(|k| (line_counts.get(*k).unwrap(), *k) );

        // TODO don't rewrite if nothing has changed since last time (on the screen)

        // display current status
        let (_twidth, theight) = termion::terminal_size().unwrap();

        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        print!("{}Lines total: {}, Unique lines: {}{}", color::Fg(color::Yellow), lines_total, line_counts.len(), style::Reset);

        // iterate over references in reverse to display top first
        for (i, key) in keys.iter().rev().enumerate() {
            if i+1 >= theight as usize { break; }

            // TODO clip to width
            print!("\n{:width$}: {}", line_counts[*key], key, width=5);
        }
    }
}
