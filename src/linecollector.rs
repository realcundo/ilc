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

    pub fn get(&self, key: &str) -> Option<usize> {
        self.line_counts.get(key).cloned()
    }

    pub fn iter(&self) -> LineCollectorResultIter {
        LineCollectorResultIter::new(&self.line_counts)
    }
}

impl Index<&str> for LineCollector {
    type Output = usize;

    fn index(&self, key: &str) -> &Self::Output {
        &self.line_counts[key]
    }
}

pub struct LineCollectorResultIter<'a> {
    sorted_lines: Vec<(usize, &'a String)>, // TODO use str
}

// TODO implemenmt IntoIterator?!
impl<'a> LineCollectorResultIter<'a> {
    fn new(line_counts: &'a HashMap<String, usize>) -> Self {
        // get all key+value pairs, swap them around and dereference the value (count)
        // so that it is copied by value. It's convenient to have count first since
        // we can then simply order by count first and string second.
        let mut sorted_lines: Vec<_> = line_counts.iter().map(|(s, c)| (*c, s)).collect();

        sorted_lines.sort_unstable();
        LineCollectorResultIter { sorted_lines }
    }
}

impl<'a> Iterator for LineCollectorResultIter<'a> {
    type Item = (usize, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        // return in reverse order so just keep popping from the vector
        self.sorted_lines.pop()
    }
}
