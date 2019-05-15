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

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience class to build LineCollector instances for testing.
    struct LineCollectorBuilder(LineCollector);

    impl LineCollectorBuilder {
        pub fn new() -> Self {
            LineCollectorBuilder(LineCollector::new())
        }

        pub fn add(mut self, line: &str) -> Self {
            self.0.insert(line.to_owned());
            self
        }

        pub fn build(self) -> LineCollector {
            self.0
        }
    }

    #[test]
    fn line_collector_is_empty_by_default() {
        let lc = LineCollectorBuilder::new().build();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 0);
        assert_eq!(lc.num_unique(), 0);
    }

    #[test]
    fn line_collector_stores_duplicate_lines() {
        let lc = LineCollectorBuilder::new()
            .add("a")
            .add("a")
            .add("a")
            .build();

        let s_a = "a".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(3, &s_a)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 3);
        assert_eq!(lc.num_unique(), 1);
    }

    #[test]
    fn line_collector_stores_unique_lines() {
        let lc = LineCollectorBuilder::new()
            .add("a")
            .add("b")
            .add("c")
            .build();

        let s_a = "a".to_string();
        let s_b = "b".to_string();
        let s_c = "c".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(1, &s_c), (1, &s_b), (1, &s_a)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 3);
        assert_eq!(lc.num_unique(), 3);
    }

    #[test]
    fn line_collector_stores_mixed_lines() {
        let lc = LineCollectorBuilder::new()
            .add("a")
            .add("b")
            .add("c")
            .add("b")
            .add("a")
            .add("b")
            .build();

        let s_a = "a".to_string();
        let s_b = "b".to_string();
        let s_c = "c".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(3, &s_b), (2, &s_a), (1, &s_c)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 6);
        assert_eq!(lc.num_unique(), 3);
    }
}
