use std::path::PathBuf;

use regex::Regex;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{self, Instant},
};

use structopt::StructOpt;
use termion::{color, style};

mod filepaths;
mod input;
mod linecollector;

use filepaths::FilePathParser;
use input::spawn_input_thread;
use linecollector::LineCollector;

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
#[structopt(no_version, name = "")]
struct Opt {
    /// Only process lines matching REGEX. Non-matching files are ignored. If
    /// the REGEX contains a capture group it will be used to process the
    /// input instead of the whole line.
    #[structopt(
        name = "REGEX",
        short = "r",
        long = "regex",
        parse(try_from_str = parse_regex)
    )]
    matching_string: Option<Regex>,

    /// Files to process. If none specified stdin is used. To specify stdin
    /// explicitly pass in "-". Directories are not supported.
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    // run the application in another function so that everything is dropped as
    // it should be before calling exit()
    match run_app() {
        Ok(_) => std::process::exit(exitcode::OK),
        Err(code) => std::process::exit(code),
    };
}

fn run_app() -> Result<(), i32> {
    let opt = Opt::from_args();

    // make sure stdout is a TTY
    if !termion::is_tty(&std::io::stdout()) {
        eprintln!("stdout is not a TTY, can't display interactively");
        return Err(exitcode::UNAVAILABLE);
    }

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
        FilePathParser::new(opt.files),
        collector.clone(),
    );

    let mut first_display_frame = true;
    let mut last_total_number_of_lines = 0;

    let mut next_forced_redraw = Instant::now();

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
            if first_display_frame
                || last_total_number_of_lines != collector.num_total()
                || Instant::now() >= next_forced_redraw
            {
                display_collected_lines(&collector);

                // refresh every 5s. Useful if the screen size changes.
                next_forced_redraw = Instant::now() + time::Duration::from_secs(5);
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
    // produced. unwrap() since we don't expect the thread to have panicked
    match input_thread.join().unwrap() {
        Ok(_) => Ok(()),
        Err(x) => {
            eprintln!("\nError: {}", x);
            Err(exitcode::IOERR)
        }
    }
}

fn display_collected_lines(line_collector: &LineCollector) {
    // print the status line
    print!(
        "{}{}{}{} total, {} unique lines{}",
        termion::cursor::Goto(1, 1),
        termion::clear::CurrentLine,
        color::Fg(color::Yellow),
        line_collector.num_total(),
        line_collector.num_unique(),
        style::Reset
    );

    // update the rest of the screen
    // unwrap panics if stdout is not a tty
    let (twidth, theight) = termion::terminal_size().unwrap();

    // XXX TODO if nothing has been printed out except the first line
    // the line is NOT printed out for some reason until "\n" is printed

    // if not enough space, don't show anything
    if theight < 2 {
        return;
    }

    // Width of the "count" field. Will be initialised when the first
    // (most frequent, i.e. the largest number) is printed out.
    let mut count_width = 0;

    // iterate over references in reverse to display top first
    // only consume (theight-1) top elements
    for (count, line) in line_collector.iter().take(theight as usize - 1) {
        // clear the line and print the count:
        if count_width == 0 {
            // we don't know the width so calculate it from the first/largest count
            count_width = (count as f32).log10().floor() as usize + 1;
        }

        print!(
            "\n{}{:width$}",
            termion::clear::CurrentLine,
            count,
            width = count_width
        );

        // print the line if there is still enough space
        if count_width + 2 < twidth as usize {
            print!(
                ": {:.width$}",
                line,
                width = twidth as usize - count_width - 2
            );
        }
    }

    print!("{}", termion::clear::AfterCursor);
}
