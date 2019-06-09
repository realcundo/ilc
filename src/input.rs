use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::thread;

use std::thread::JoinHandle;

use regex::Regex;

use crate::linecollector;

pub fn spawn_input_thread(
    regex: Option<Regex>,
    files: Vec<PathBuf>,
    line_collector: std::sync::Arc<std::sync::Mutex<linecollector::LineCollector>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // go over input files and process them one by one
        if files.is_empty() {
            // no input files, process stdin only
            let input_buffer = io::stdin();
            let mut input_reader = input_buffer.lock();

            process_file(&mut input_reader, &regex, &line_collector);
            return;
        } else {
            for filename in files {
                // simply panic if file can't be opened
                // XXX better error message! But it gets overwritten by the display thread
                // XXX mut not needed?! (according to docs but read_line seems to need it)
                let input_file = File::open(filename).unwrap();
                let mut input_reader = BufReader::new(input_file);
                process_file(&mut input_reader, &regex, &line_collector);
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

        // match the whole line if there is no regex, otherwise use the regex to match&extract the value.
        // Use the last capture group (there is either 1 or 2 capture groups)
        let s = match regex {
            None => input_line.trim_end(),
            Some(re) => match re.captures(&input_line) {
                None => continue, // no match, go to the next line
                Some(captures) => captures.get(captures.len() - 1).unwrap().as_str(), // XXX this is a copy since captures doesn't live longer than s
            },
        };

        // insert the line into the collector. The collector adopts the line.
        {
            line_collector.lock().unwrap().insert(&s);
        }
    }
}
