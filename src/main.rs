extern crate termion;

use std::io::{self, BufRead};
use std::time::Instant;

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

    let mut tick_last_displayed: Option<Instant> = None;

    loop {
        input_line.clear();
        match input_reader.read_line(&mut input_line) {
            Ok(0) => break,     // quit the loop, EOF
            Ok(_) => {}         // process the line
            Err(_) => continue, // ignore the line and read the next one. Line is probably non-utf8
        }

        // insert the line into the collector. The collector adopts the line.
        collector.insert(&input_line.trim_end());

        // clear the screen the first time, subsequently we make sure we call clear::UntilNewLine for each line
        // this reduces the flicker since each character is modified only once per loop
        if !has_cleared_screen {
            has_cleared_screen = true;
            print!("{}", termion::clear::All);
        }

        // display current status if enough time has elapsed. Displaying after each line is very expensive
        let tick_now = Instant::now();
        if match tick_last_displayed {
            None => false,
            Some(tick) => tick_now.duration_since(tick).as_millis() < 5,
        } {
            continue;
        }

        tick_last_displayed = Some(tick_now);

        display_collected_lines(&collector);
    }

    // display it again in case we ended before we could display everything
    // if we have no lines, don't display anything
    if collector.num_total() > 0 {
        display_collected_lines(&collector);

        // insert newline after processing all lines
        println!();
    }
}

fn display_collected_lines(line_collector: &LineCollector) {
    // print the status line
    print!(
        "{}{}Lines total: {}, Unique lines: {}{}",
        termion::cursor::Goto(1, 1),
        color::Fg(color::Yellow),
        line_collector.num_total(),
        line_collector.num_unique(),
        style::Reset
    );

    // update the rest of the screen
    let (twidth, theight) = termion::terminal_size().unwrap();

    // if not enough space, don't show anything
    if theight < 2 {
        return;
    }

    // iterate over references in reverse to display top first
    // only consume (theight-1) top elements
    for (count, line) in line_collector.iter().take(theight as usize - 1) {
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
