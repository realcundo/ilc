extern crate termion;

use std::io::{self, BufRead};
use termion::{color, style};

mod linecollector;
use linecollector::LineCollector;

fn main() {
    // keeps track of how often each line has occurred
    let mut collector = LineCollector::new();
    let mut has_cleared_screen = false;

    let input_buffer = io::stdin();
    let mut input_reader = input_buffer.lock();
    let mut input_line = String::new();

    loop {
        input_line.clear();
        match input_reader.read_line(&mut input_line) {
            Ok(0) => break,     // quit the loop, EOF
            Ok(_) => {}         // process the line
            Err(_) => continue, // ignore the line and read the next one. Line is probably non-utf8
        }

        // insert the line into the collector. The collector adopts the line.
        collector.insert(&input_line.trim_end());

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
            collector.num_total(),
            collector.num_unique(),
            style::Reset
        );

        // if not enough space, don't show anything
        if theight < 2 {
            continue;
        }

        // iterate over references in reverse to display top first
        // only consume (theight-1) top elements
        for (count, line) in collector.iter().take(theight as usize - 1) {
            // render the full output line
            let out_line = format!("{:width$}: {}", count, line, width = 5);

            let mut out_chars: Vec<char> = out_line.chars().collect();
            out_chars.truncate(twidth as usize);

            let out_line: String = out_chars.into_iter().collect();

            // clip the line to terminal width
            //print!("\n{}{}", out_line, termion::clear::UntilNewline);
            print!("\n{}{}", termion::clear::CurrentLine, out_line);
        }
    }

    // insert newline after processing all lines
    println!();
}
