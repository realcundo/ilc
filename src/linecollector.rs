use std::collections::HashMap;
use std::ops::Index;

#[derive(Default, Debug)]
pub struct LineCollector {
    line_counts: HashMap<String, usize>,
    total_lines: usize,
}

impl LineCollector {
    pub fn new() -> Self {
        LineCollector {
            ..Default::default()
        }
    }

    pub fn num_total(&self) -> usize {
        self.total_lines
    }

    pub fn num_unique(&self) -> usize {
        self.line_counts.len()
    }

    pub fn insert(&mut self, line: String) {
        self.total_lines += 1;

        let line_count = self.line_counts.entry(line).or_insert(0);
        *line_count += 1;
    }

    // TODO why &usize and not usize?
    pub fn get(&self, key: &str) -> Option<&usize> {
        self.line_counts.get(key)
    }

    pub fn iter(&self) -> LineCollectorResultIter {
        LineCollectorResultIter::new(&self.line_counts)
    }
}

impl Index<&str> for LineCollector {
    type Output = usize;

    // TODO why &usize and not usize?
    // and why is Ouput without &?
    fn index(&self, key: &str) -> &usize {
        self.get(key).unwrap()
    }
}

pub struct LineCollectorResultIter<'a> {
    sorted_lines: Vec<&'a String>, // TODO use str
}

// TODO implemenmt IntoIterator?!
impl<'a> LineCollectorResultIter<'a> {
    fn new(line_counts: &'a HashMap<String, usize>) -> Self {
        let mut sorted_lines: Vec<_> = line_counts.keys().collect();
        sorted_lines.sort_unstable_by_key(|k| (line_counts[*k], *k));
        LineCollectorResultIter { sorted_lines }
    }
}

impl<'a> Iterator for LineCollectorResultIter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<&'a String> {
        // return in reverse order so just keep popping from vector
        self.sorted_lines.pop()
    }
}
