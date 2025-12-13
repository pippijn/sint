use sint_core::small_map::SmallMap;

#[test]
fn test_small_map_ordering() {
    let mut map = SmallMap::new();
    map.insert(2u32, "C");
    map.insert(0u32, "A");
    map.insert(1u32, "B");

    // Keys should be sorted 0, 1, 2
    let keys: Vec<u32> = map.keys().collect();
    assert_eq!(keys, vec![0, 1, 2]);

    // Values should be sorted by key ("A", "B", "C")
    let values: Vec<&str> = map.values().cloned().collect();
    assert_eq!(values, vec!["A", "B", "C"]);

    // Iteration should be sorted
    let pairs: Vec<(u32, &str)> = map.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(pairs, vec![(0, "A"), (1, "B"), (2, "C")]);
}

#[test]
fn test_small_map_sparse() {
    let mut map = SmallMap::new();
    map.insert(0u32, "A");
    map.insert(2u32, "C");

    let keys: Vec<u32> = map.keys().collect();
    assert_eq!(keys, vec![0, 2]);
}
