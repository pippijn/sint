use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// Trait for types that have a unique identifier.
pub trait Identifiable {
    type Id: Eq
        + PartialEq
        + Clone
        + Ord
        + Serialize
        + for<'de> Deserialize<'de>
        + JsonSchema
        + Debug;
    fn id(&self) -> &Self::Id;
}

/// A map-like structure optimized for tiny collections where keys are fields of the value.
/// Uses a `Vec<V>` internally and performs linear lookups.
/// Serializes as a BTreeMap (map) to ensure compatibility with clients.
#[derive(Clone, PartialEq, Eq, Debug, JsonSchema)]
pub struct FieldMap<V>
where
    V: Identifiable + JsonSchema,
{
    #[schemars(with = "BTreeMap<V::Id, V>")]
    data: Vec<V>,
}

impl<V> Default for FieldMap<V>
where
    V: Identifiable + JsonSchema,
{
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<V> FieldMap<V>
where
    V: Identifiable + JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: V) -> Option<V> {
        let id = value.id();
        let res = if let Some(pos) = self.data.iter().position(|x| x.id() == id) {
            Some(std::mem::replace(&mut self.data[pos], value))
        } else {
            self.data.push(value);
            None
        };
        // Keep it sorted by ID for determinism (matching BTreeMap)
        self.data.sort_by(|a, b| a.id().cmp(b.id()));
        res
    }

    pub fn get<Q>(&self, id: &Q) -> Option<&V>
    where
        V::Id: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.data.iter().find(|x| x.id().borrow() == id)
    }

    pub fn get_mut<Q>(&mut self, id: &Q) -> Option<&mut V>
    where
        V::Id: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.data.iter_mut().find(|x| x.id().borrow() == id)
    }

    pub fn remove<Q>(&mut self, id: &Q) -> Option<V>
    where
        V::Id: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        if let Some(pos) = self.data.iter().position(|x| x.id().borrow() == id) {
            Some(self.data.remove(pos))
        } else {
            None
        }
    }

    pub fn contains_key<Q>(&self, id: &Q) -> bool
    where
        V::Id: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.data.iter().any(|x| x.id().borrow() == id)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &V::Id> {
        self.data.iter().map(|v| v.id())
    }
}

// Implement Index trait for direct access (panics if missing)
impl<V, Q> Index<&Q> for FieldMap<V>
where
    V: Identifiable + JsonSchema,
    V::Id: Borrow<Q>,
    Q: Eq + ?Sized,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<V, Q> IndexMut<&Q> for FieldMap<V>
where
    V: Identifiable + JsonSchema,
    V::Id: Borrow<Q>,
    Q: Eq + ?Sized,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).expect("no entry found for key")
    }
}

// Custom Serialization to match BTreeMap behavior (Map)
impl<V> Serialize for FieldMap<V>
where
    V: Identifiable + JsonSchema + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for v in &self.data {
            map.serialize_entry(v.id(), v)?;
        }
        map.end()
    }
}

// Custom Deserialization
impl<'de, V> Deserialize<'de> for FieldMap<V>
where
    V: Identifiable + JsonSchema + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = BTreeMap::<V::Id, V>::deserialize(deserializer)?;
        let mut fm = FieldMap::new();
        // Insert values. Note: order might change based on BTreeMap iteration
        for (_, v) in map {
            fm.insert(v);
        }
        Ok(fm)
    }
}

// Implement IntoIterator for owned iteration
impl<V> IntoIterator for FieldMap<V>
where
    V: Identifiable + JsonSchema,
{
    type Item = (V::Id, V);
    type IntoIter = std::iter::Map<std::vec::IntoIter<V>, fn(V) -> (V::Id, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter().map(|v| (v.id().clone(), v))
    }
}

// Implement IntoIterator for ref iteration
impl<'a, V> IntoIterator for &'a FieldMap<V>
where
    V: Identifiable + JsonSchema,
{
    type Item = (&'a V::Id, &'a V);
    type IntoIter = std::iter::Map<std::slice::Iter<'a, V>, fn(&'a V) -> (&'a V::Id, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().map(|v| (v.id(), v))
    }
}
