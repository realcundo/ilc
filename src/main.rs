#[macro_use]
extern crate structopt;

extern crate regex;
extern crate termion;

use std::path::PathBuf;
use structopt::StructOpt;

use regex::Regex;

use std::io::{self, BufRead, BufReader};

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time};

use termion::{color, style};

mod linecollector;
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
#[structopt(name = "", version = "", author = "")]
struct Opt {
    /// Only process lines matching REGEX. Non-matching files are ignored. If the REGEX
    /// contains a capture group it will be used to process the input instead of the whole line.
    #[structopt(
        name = "REGEX",
        short = "r",
        long = "regex",
        parse(try_from_str = "parse_regex")
    )]
    matching_string: Option<Regex>,

    /// Files to process. If none specified stdin is used. To specify stdin explicitly pass in "-".
    /// Directories are not supported.
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
    let _input_thread = spawn_input_thread(collector.clone(), opt.clone());

    let mut first_display_frame = true;
    let mut last_total_number_of_lines = 0;

    // keep displaying this in a loop for as long as the input thread is running
    while Arc::strong_count(&collector) > 1 {
        // clear the screen the first time, subsequently we make sure we call clear::UntilNewLine for each line
        // this reduces the flicker since each character is modified only once per loop
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
}

fn spawn_input_thread(
    line_collector: std::sync::Arc<std::sync::Mutex<linecollector::LineCollector>>,
    opt: Opt,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // go over input files and process them one by one
        if opt.files.is_empty() {
            // no input files, process stdin only
            let input_buffer = io::stdin();
            let mut input_reader = input_buffer.lock();

            process_file(&mut input_reader, &opt.matching_string, &line_collector);
            return;
        } else {
            for filename in opt.files {
                // simply panic if file can't be opened
                // XXX better error message! But it gets overwritten by the display thread
                // XXX mut not needed?! (according to docs but read_line seems to need it)
                let input_file = File::open(filename).unwrap();
                let mut input_reader = BufReader::new(input_file);
                process_file(&mut input_reader, &opt.matching_string, &line_collector);
            }
        }

        // no more input, quit the thread, releasing the Arc
    })
}

fn process_file(
    input_stream: &mut impl io::BufRead,
    regex: &Option<Regex>,
    line_collector: &std::sync::Mutex<linecollector::LineCollector>,
) {
    let mut input_line = String::new();

    // main input loop to process input_stream
    loop {
        input_line.clear();
        match input_stream.read_line(&mut input_line) {
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

        let mut out_chars: Vec<char> = out_line.chars().collect();
        out_chars.truncate(twidth as usize);

        let out_line: String = out_chars.into_iter().collect();

        // clip the line to terminal width
        //print!("\n{}{}", out_line, termion::clear::UntilNewline);
        print!("\n{}{}", termion::clear::CurrentLine, out_line);
    }
}
