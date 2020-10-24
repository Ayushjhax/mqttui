use crate::topic;
use std::collections::HashSet;

pub fn get_shown_topics<'a>(existing: &'a [String], opened: &HashSet<String>) -> Vec<&'a str> {
    let all = build_all_tree_variants(existing);
    filter_topics_by_opened(&all, opened)
}

pub fn build_all_tree_variants<'a>(existing: &'a [String]) -> Vec<&'a str> {
    let mut result = Vec::new();

    for entry in existing {
        for parent in topic::get_all_parents(entry) {
            result.push(parent);
        }

        result.push(entry);
    }

    result.sort_unstable();
    result.dedup();

    result
}

pub fn filter_topics_by_opened<'a>(all: &[&'a str], opened: &HashSet<String>) -> Vec<&'a str> {
    let mut shown = Vec::new();

    for entry in all {
        if is_topic_opened(opened, entry) {
            shown.push(entry.to_owned());
        }
    }

    shown
}

fn is_topic_opened(opened: &HashSet<String>, topic: &str) -> bool {
    topic::get_all_parents(topic)
        .iter()
        .cloned()
        .all(|t| opened.contains(t))
}

#[test]
fn tree_variants_empty_stays_empty() {
    let actual = build_all_tree_variants(&[]);
    assert_eq!(0, actual.len());
}

#[test]
fn tree_variants_shortest_path() {
    let topics = ["foo".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["foo"]);
}

#[test]
fn tree_variants_path_gets_splitted() {
    let topics = ["foo/bar".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["foo", "foo/bar"]);
}

#[test]
fn tree_variants_dont_duplicate() {
    let topics = ["a/b".to_owned(), "a/b/c".to_owned(), "a/d".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["a", "a/b", "a/b/c", "a/d"]);
}

#[cfg(test)]
const ALL_EXAMPLES: [&str; 10] = [
    "a",
    "a/b",
    "a/b/c",
    "a/d",
    "e",
    "e/f",
    "e/f/g",
    "e/f/g/h",
    "e/f/g/h/i",
    "e/j",
];

#[test]
fn filter_topics_by_opened_shows_only_top_level() {
    let opened = HashSet::new();
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_some() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ["a", "a/b", "a/d", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_only_when_all_parents_are_opened() {
    let mut opened = HashSet::new();
    opened.insert("a/b".to_string());
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_all() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    opened.insert("a/b".to_string());
    opened.insert("e".to_string());
    opened.insert("e/f".to_string());
    opened.insert("e/f/g".to_string());
    opened.insert("e/f/g/h".to_string());

    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ALL_EXAMPLES);
}
