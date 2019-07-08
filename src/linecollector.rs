use lazysort::SortedBy;
use std::{cmp::Ordering::Equal, collections::HashMap, ops::Index};

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

    // Inserts the line into collection. Makes a copy of the input
    // and takes ownership of the line.
    pub fn insert(&mut self, line: &str) {
        self.total_lines += 1;

        let line_count = self.line_counts.entry(line.to_owned()).or_insert(0);
        *line_count += 1;
    }

    pub fn get(&self, key: &str) -> Option<usize> {
        self.line_counts.get(key).cloned()
    }

    #[cfg(feature = "slow_linecollector")]
    pub fn iter(&self) -> LineCollectorResultIter {
        LineCollectorResultIter::new(&self.line_counts)
    }

    #[cfg(not(feature = "slow_linecollector"))]
    pub fn iter(&self) -> impl Iterator<Item = (usize, &String)> {
        self.line_counts
            .iter()
            // swap count+string around and dereference count so it's copied
            .map(|(s, c)| (*c, s))
            // order by count desc, then by name asc
            .sorted_by(|(c1, s1), (c2, s2)| match c2.cmp(&c1) {
                Equal => s1.cmp(s2),
                x => x,
            })
    }

    /*
    //pub fn mytest(before: &Vec<&str>) -> impl Iterator<Item  = &&str> {
    pub fn test<'vec, 'item>(&self, before: &'vec Vec<&'item str>) -> impl Iterator<Item = &'vec &'item str> +'vec {
        before.iter().sorted_by(|a, b| {
            match a.len().cmp(&b.len()) {
                Equal => a.cmp(b),
                x => x
            }
        })
    }
    */
}

impl Index<&str> for LineCollector {
    type Output = usize;

    fn index(&self, key: &str) -> &Self::Output {
        &self.line_counts[key]
    }
}

#[cfg(feature = "slow_linecollector")]
pub struct LineCollectorResultIter<'a> {
    sorted_lines: Vec<(usize, &'a String)>, // TODO use str
}

// TODO implemenmt IntoIterator?!
#[cfg(feature = "slow_linecollector")]
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

#[cfg(feature = "slow_linecollector")]
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

    #[test]
    fn line_collector_is_empty_by_default() {
        let lc = LineCollector::new();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 0);
        assert_eq!(lc.num_unique(), 0);
    }

    #[test]
    fn line_collector_stores_duplicate_lines() {
        let mut lc = LineCollector::new();
        lc.insert("a");
        lc.insert("a");
        lc.insert("a");

        let s_a = "a".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(3, &s_a)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 3);
        assert_eq!(lc.num_unique(), 1);
    }

    #[test]
    fn line_collector_stores_unique_lines() {
        let mut lc = LineCollector::new();
        lc.insert("a");
        lc.insert("b");
        lc.insert("c");

        let s_a = "a".to_string();
        let s_b = "b".to_string();
        let s_c = "c".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(1, &s_a), (1, &s_b), (1, &s_c)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 3);
        assert_eq!(lc.num_unique(), 3);
    }

    #[test]
    fn duplicate_lines_are_ordered_lexicographically() {
        let mut lc = LineCollector::new();
        lc.insert("a");
        lc.insert("c");
        lc.insert("b");
        lc.insert("b");
        lc.insert("c");
        lc.insert("a");

        let s_a = "a".to_string();
        let s_b = "b".to_string();
        let s_c = "c".to_string();

        let returned_items: Vec<_> = lc.iter().collect();
        let expected_items = vec![(2, &s_a), (2, &s_b), (2, &s_c)];
        assert_eq!(returned_items, expected_items);

        assert_eq!(lc.num_total(), 6);
        assert_eq!(lc.num_unique(), 3);
    }

    #[test]
    fn line_collector_stores_mixed_lines() {
        let mut lc = LineCollector::new();
        lc.insert("a");
        lc.insert("b");
        lc.insert("c");
        lc.insert("b");
        lc.insert("a");
        lc.insert("b");

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
