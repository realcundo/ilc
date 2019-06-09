extern crate termion;

use std::io::{self, BufRead};

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time};

use termion::{color, style};

mod linecollector;
use linecollector::LineCollector;

// TODO add docs: https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html

fn main() {
    // keeps track of how often each line has occurred.
    // The number of clones is important as well,
    // if the number drops to one the main printing loop will
    // finish and the program will exit. It's a hacky way
    // for the reading thread to signal the main thread that
    // there is no more input.
    let collector = Arc::new(Mutex::new(LineCollector::new()));

    // create a second Rc to the collector which is used in the thread
    // and released when the thread stops (when there is no more
    // input)
    let _input_thread = spawn_input_thread(collector.clone());

    let mut has_cleared_screen = false;

    let mut last_total_number_of_lines = None;

    // keep displaying this in a loop for as long as the input thread is running
    while Arc::strong_count(&collector) > 1 {
        // clear the screen the first time, subsequently we make sure we call clear::UntilNewLine for each line
        // this reduces the flicker since each character is modified only once per loop
        if !has_cleared_screen {
            has_cleared_screen = true;
            print!("{}", termion::clear::All);
        }

        {
            let collector = collector.lock().unwrap();

            // only redraw the screen when total number of lines has changed
            let current_total_number_of_lines = Some(collector.num_total());
            if current_total_number_of_lines != last_total_number_of_lines {
                display_collected_lines(&collector);
                last_total_number_of_lines = current_total_number_of_lines;
            }
        }

        // refresh after a while
        thread::sleep(time::Duration::from_millis(50));
    }

    // display it again in case we ended before we could display everything
    // if we have no lines, don't display anything
    let collector = collector.lock().unwrap();
    if collector.num_total() > 0 {
        display_collected_lines(&collector);

        // insert newline after processing all lines
        println!();
    }
}

fn spawn_input_thread(
    line_collector: std::sync::Arc<std::sync::Mutex<linecollector::LineCollector>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let input_buffer = io::stdin();
        let mut input_reader = input_buffer.lock();
        let mut input_line = String::new();

        // main input loop
        loop {
            input_line.clear();
            match input_reader.read_line(&mut input_line) {
                Ok(0) => break,     // quit the loop, EOF
                Ok(_) => {}         // process the line
                Err(_) => continue, // ignore the line and read the next one. Line is probably non-utf8
            }

            // insert the line into the collector. The collector adopts the line.
            {
                line_collector
                    .lock()
                    .unwrap()
                    .insert(&input_line.trim_end());
            }
        }

        // no more input, quit the thread, releasing the Arc
    })
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
