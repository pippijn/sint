use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sint_core::small_map::SmallMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
struct TestValue(u32);

#[test]
fn test_small_map_basic_operations() {
    let mut map: SmallMap<u32, TestValue> = SmallMap::new();

    // Test insertion
    assert!(map.insert(0, TestValue(100)).is_none());
    assert!(map.insert(5, TestValue(500)).is_none());
    assert_eq!(map.len(), 2);

    // Test retrieval
    assert_eq!(map.get(&0), Some(&TestValue(100)));
    assert_eq!(map.get(&5), Some(&TestValue(500)));
    assert_eq!(map.get(&2), None);

    // Test update
    assert_eq!(map.insert(0, TestValue(101)), Some(TestValue(100)));
    assert_eq!(map.get(&0), Some(&TestValue(101)));

    // Test removal
    assert_eq!(map.remove(&5), Some(TestValue(500)));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&5), None);
}

#[test]
fn test_small_map_iteration() {
    let mut map: SmallMap<u32, TestValue> = SmallMap::new();
    map.insert(1, TestValue(10));
    map.insert(3, TestValue(30));
    map.insert(2, TestValue(20));

    // Keys should be returned in order because of the underlying structure
    let keys: Vec<u32> = map.keys().collect();
    assert_eq!(keys, vec![1, 2, 3]);

    let values: Vec<u32> = map.values().map(|v| v.0).collect();
    assert_eq!(values, vec![10, 20, 30]);
}

#[test]
fn test_small_map_serialization() {
    let mut map: SmallMap<u32, TestValue> = SmallMap::new();
    map.insert(1, TestValue(10));
    map.insert(2, TestValue(20));

    let json = serde_json::to_string(&map).unwrap();
    // Verify it serializes as a JSON map for client compatibility
    assert!(json.contains("\"1\":10"));
    assert!(json.contains("\"2\":20"));

    let deserialized: SmallMap<u32, TestValue> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.get(&1), Some(&TestValue(10)));
    assert_eq!(deserialized.get(&2), Some(&TestValue(20)));
}

#[test]
fn test_small_map_overflow() {
    // Capacity is 16. Test what happens when we go beyond it.
    let mut map: SmallMap<u32, TestValue> = SmallMap::new();
    for i in 0..20 {
        map.insert(i, TestValue(i));
    }

    assert_eq!(map.len(), 20);
    assert_eq!(map.get(&19), Some(&TestValue(19)));

    // Removal from overflowed map
    assert_eq!(map.remove(&19), Some(TestValue(19)));
    assert_eq!(map.len(), 19);
}

#[test]
fn test_small_map_index_traits() {
    let mut map: SmallMap<u32, TestValue> = SmallMap::new();
    map.insert(1, TestValue(10));

    assert_eq!(map[1], TestValue(10));
    map[1] = TestValue(11);
    assert_eq!(map[1], TestValue(11));
}

#[test]
#[should_panic(expected = "no entry found for key")]
fn test_small_map_index_panic() {
    let map: SmallMap<u32, TestValue> = SmallMap::new();
    let _ = map[1];
}

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
