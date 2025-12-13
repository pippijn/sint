use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// Trait for types that can be used as indices in a SmallMap.
/// Must be convertible to/from usize and be a valid small integer.
pub trait SmallIndex:
    Copy + From<u8> + Ord + Eq + std::hash::Hash + std::fmt::Debug + 'static
{
    fn from_usize(idx: usize) -> Self;
    fn to_usize(self) -> usize;
}

impl SmallIndex for u32 {
    fn from_usize(idx: usize) -> Self {
        idx as u32
    }
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl SmallIndex for usize {
    fn from_usize(idx: usize) -> Self {
        idx
    }
    fn to_usize(self) -> usize {
        self
    }
}

/// A map implementation optimized for small, dense integer keys.
/// Uses a `Vec<Option<V>>` internally but presents a map-like interface.
/// Serializes as a BTreeMap (map) to ensure compatibility with clients.
#[derive(Clone, PartialEq, Eq, Debug, JsonSchema)]
pub struct SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    #[schemars(with = "BTreeMap<K, V>")]
    data: Vec<Option<V>>,
    _marker: PhantomData<K>,
}

impl<K, V> Default for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    fn default() -> Self {
        Self {
            data: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<K, V> SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let idx = key.to_usize();
        if idx >= self.data.len() {
            self.data.resize_with(idx + 1, || None);
        }
        std::mem::replace(&mut self.data[idx], Some(value))
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let idx: usize = (*key).to_usize();
        self.data.get(idx).and_then(|opt| opt.as_ref())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let idx: usize = (*key).to_usize();
        self.data.get_mut(idx).and_then(|opt| opt.as_mut())
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let idx: usize = (*key).to_usize();
        if idx < self.data.len() {
            self.data[idx].take()
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let idx: usize = (*key).to_usize();
        if idx < self.data.len() {
            self.data[idx].is_some()
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.data.iter().filter(|x| x.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Iterator over (Key, &Value).
    /// Note: Returns `K` by value, unlike BTreeMap which returns `&K`.
    /// K is Copy, so this is usually fine.
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.data.iter().enumerate().filter_map(|(i, v)| {
            v.as_ref().map(|val| (K::from_usize(i), val))
        })
    }

    /// Mutable iterator over (Key, &mut Value).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.data.iter_mut().enumerate().filter_map(|(i, v)| {
            v.as_mut().map(|val| (K::from_usize(i), val))
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().filter_map(|x| x.as_ref())
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().filter_map(|x| x.as_mut())
    }

    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.data.iter().enumerate().filter_map(|(i, v)| {
            if v.is_some() {
                Some(K::from_usize(i))
            } else {
                None
            }
        })
    }
}

// Implement Index trait for direct access (panics if missing, like BTreeMap/HashMap)
impl<K, V> Index<K> for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index).expect("no entry found for key")
    }
}

impl<K, V> Index<&K> for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<K, V> IndexMut<K> for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(&index).expect("no entry found for key")
    }
}

impl<K, V> IndexMut<&K> for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    fn index_mut(&mut self, index: &K) -> &mut Self::Output {
        self.get_mut(index).expect("no entry found for key")
    }
}

// Custom Serialization to match BTreeMap behavior (Map)
impl<K, V> Serialize for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: Serialize + JsonSchema,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let len = self.len();
        let mut map = serializer.serialize_map(Some(len))?;
        for (i, val) in self.data.iter().enumerate() {
            if let Some(v) = val {
                let key = K::from_usize(i);
                map.serialize_entry(&key, v)?;
            }
        }
        map.end()
    }
}

// Custom Deserialization
impl<'de, K, V> Deserialize<'de> for SmallMap<K, V>
where
    K: SmallIndex + Serialize + Deserialize<'de>,
    V: Deserialize<'de> + JsonSchema,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = BTreeMap::<K, V>::deserialize(deserializer)?;
        let mut sm = SmallMap::new();
        for (k, v) in map {
            sm.insert(k, v);
        }
        Ok(sm)
    }
}

// Implement IntoIterator for owned iteration
impl<K, V> IntoIterator for SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    type Item = (K, V);
    type IntoIter = std::iter::FilterMap<
        std::iter::Enumerate<std::vec::IntoIter<Option<V>>>,
        fn((usize, Option<V>)) -> Option<(K, V)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .into_iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|val| (K::from_usize(i), val)))
    }
}

// Implement IntoIterator for ref iteration
impl<'a, K, V> IntoIterator for &'a SmallMap<K, V>
where
    K: SmallIndex + Serialize,
    V: JsonSchema,
{
    type Item = (K, &'a V);
    type IntoIter = std::iter::FilterMap<
        std::iter::Enumerate<std::slice::Iter<'a, Option<V>>>,
        fn((usize, &'a Option<V>)) -> Option<(K, &'a V)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().enumerate().filter_map(|(i, v)| {
            v.as_ref().map(|val| (K::from_usize(i), val))
        })
    }
}
