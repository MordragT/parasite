use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedMap<K, V> {
    raw: Vec<(K, V)>,
}

impl<K, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self { raw: Vec::new() }
    }
}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Ord,
{
    pub fn insert(&mut self, key: K, value: V) -> usize {
        match self.raw.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(i) => i,
            Err(i) => {
                self.raw.insert(i, (key, value));
                i
            }
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.raw.binary_search_by(|(k, _)| k.cmp(key)).is_ok()
    }
}

impl<K, V> FromIterator<(K, V)> for OrderedMap<K, V>
where
    K: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut raw: Vec<(K, V)> = iter.into_iter().collect();
        raw.sort_by(|(a, _), (b, _)| a.cmp(b));
        raw.dedup_by(|(a, _), (b, _)| a == b);
        Self { raw }
    }
}

impl<K, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = std::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<K, V> Deref for OrderedMap<K, V> {
    type Target = [(K, V)];

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<K, V> Extend<(K, V)> for OrderedMap<K, V>
where
    K: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        self.raw.extend(iter);
        self.raw.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        self.raw.dedup_by(|(a, _), (b, _)| a == b);
    }
}

/// An ordered set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedSet<T> {
    raw: Vec<T>,
}

impl<T> OrderedSet<T> {
    pub fn new() -> Self {
        Self { raw: Vec::new() }
    }
}

impl<T> Default for OrderedSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> OrderedSet<T>
where
    T: Ord,
{
    pub fn insert(&mut self, item: T) -> usize {
        match self.raw.binary_search(&item) {
            Ok(i) => i,
            Err(i) => {
                self.raw.insert(i, item);
                i
            }
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.raw.binary_search(item).is_ok()
    }
}

impl<T> FromIterator<T> for OrderedSet<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut raw: Vec<T> = iter.into_iter().collect();
        raw.sort();
        raw.dedup();
        Self { raw }
    }
}

impl<T> IntoIterator for OrderedSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<T> Deref for OrderedSet<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<T> Extend<T> for OrderedSet<T>
where
    T: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.raw.extend(iter);
        self.raw.sort_unstable();
        self.raw.dedup();
    }
}
