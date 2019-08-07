use std::{
    io::{self, BufReader},
    thread::{self, JoinHandle},
};

use regex::Regex;

// use ex instead of std for better error messages
use ex::fs::File;

use crate::{filepaths::FilePathParser, linecollector};

/// Spawn a new thread that reads the contents of `files` and feed lines to
/// `line_collector` if `regex` matches. If `files` is empty read the `stdin`.
/// Termninate the thread on first io error.
pub fn spawn_input_thread(
    regex: Option<Regex>,
    input_files: FilePathParser,
    line_collector: std::sync::Arc<std::sync::Mutex<linecollector::LineCollector>>,
) -> JoinHandle<ex::io::Result<()>> {
    thread::spawn(move || {
        // go over input files and process them one by one
        for filename in input_files.files {
            // return if file can't be opened
            let input_file = File::open(filename)?;
            let mut input_reader = BufReader::new(input_file);
            process_file(&mut input_reader, &regex, &line_collector);
        }

        // process stdin as last
        if input_files.has_stdin {
            // no input files, process stdin only
            let input_buffer = io::stdin();
            let mut input_reader = input_buffer.lock();

            process_file(&mut input_reader, &regex, &line_collector);
        }

        // no more input, quit the thread, releasing the Arc
        Ok(())
    })
}

/// Consume the `input_stream` and the lines to `line_collector` if `regex`
/// matches. This function block until all input is consumed or an error occurs.
/// Returns nothing.
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

        // match the whole line if there is no regex, otherwise use the regex to
        // match&extract the value. Use the last capture group (there is either
        // 1 or 2 capture groups)
        let s = match regex {
            None => input_line.trim_end(),
            Some(re) => match re.captures(&input_line) {
                None => continue, // no match, go to the next line
                // XXX this is a copy since captures don't live longer than s. Maybe can be fixed?
                Some(captures) => captures.get(captures.len() - 1).unwrap().as_str(),
            },
        };

        // insert the line into the collector. The collector adopts the line.
        {
            line_collector.lock().unwrap().insert(&s);
        }
    }
}
