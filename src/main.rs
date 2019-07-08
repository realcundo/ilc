use std::path::PathBuf;
use structopt::StructOpt;

use regex::Regex;

use std::{
    sync::{Arc, Mutex},
    thread, time,
};

use termion::{color, style};

mod input;
mod linecollector;

use linecollector::LineCollector;

use input::spawn_input_thread;

fn parse_regex(src: &str) -> Result<Regex, regex::Error> {
    let re = Regex::new(src)?;
    match re.captures_len() {
        1 | 2 => Ok(re),
        _ => Err(regex::Error::Syntax(
            "At most one capture group can be specified".to_string(),
        )),
    }
}

// TODO add docs: https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html

/// Tool to interactively display most common matching lines.
///
/// The primary use case is to read stdin from a stream, filter the lines
/// using a regular expression and periodically display top most common lines.
///
/// Input files (and stdin) are opened and processed in sequence.
///
/// See https://docs.rs/regex/#syntax for regex syntax.
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "", version = "", author = "")]
struct Opt {
    /// Only process lines matching REGEX. Non-matching files are ignored. If
    /// the REGEX contains a capture group it will be used to process the
    /// input instead of the whole line.
    #[structopt(
        name = "REGEX",
        short = "r",
        long = "regex",
        parse(try_from_str = "parse_regex")
    )]
    matching_string: Option<Regex>,

    /// Files to process. If none specified stdin is used. To specify stdin
    /// explicitly pass in "-". Directories are not supported.
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

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
    let input_thread = spawn_input_thread(
        opt.matching_string.clone(),
        opt.files.clone(),
        collector.clone(),
    );

    let mut first_display_frame = true;
    let mut last_total_number_of_lines = 0;

    // keep displaying this in a loop for as long as the input thread is running
    while Arc::strong_count(&collector) > 1 {
        // clear the screen the first time, subsequently we make sure we call
        // clear::UntilNewLine for each line this reduces the flicker since each
        // character is modified only once per loop
        if first_display_frame {
            print!("{}", termion::clear::All);
        }

        {
            let collector = collector.lock().unwrap();

            // only redraw the screen when total number of lines has changed
            if first_display_frame || last_total_number_of_lines != collector.num_total() {
                display_collected_lines(&collector);
            }

            last_total_number_of_lines = collector.num_total();
        }

        first_display_frame = false;

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

    // the thread is done so get its result and print any error that might've been
    // produced XXX unwrap() since we don't expect the thread to have panicked
    // XXX use correct return code when exitting the process
    match input_thread.join().unwrap() {
        Ok(_) => (),
        Err(x) => println!("\nError: {}", x),
    };
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

    // XXX TODO if nothing has been printed out except the first line
    // the line is NOT printed out for some reason until "\n" is printed

    // if not enough space, don't show anything
    if theight < 2 {
        return;
    }

    // iterate over references in reverse to display top first
    // only consume (theight-1) top elements
    for (count, line) in line_collector.iter().take(theight as usize - 1) {
        // render the full output line
        let out_line = format!("{:width$}: {}", count, line, width = 5);

        // clear currrent line and print the trimmed string
        // printing and the clearing the rest of the line would be more efficient but
        // termion::clear::UntilNewline seems to leave artefacts
        print!(
            "\n{}{:.width$}",
            termion::clear::CurrentLine,
            out_line,
            width = twidth as usize
        );
    }
}
