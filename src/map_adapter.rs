/// Adapter that wraps a `Vec<T>` and provides a dict-like interface where
/// vector indices serve as keys.
///
/// `MapAdapter` bridges the gap between array-based and map-based graph
/// representations.  It lets a `Vec<T>` behave as a mapping from `usize` to
/// `T`, which is exactly what is needed to use array-backed storage with the
/// generic [`Graph`](crate::Graph) trait.
///
/// This is a port of the identical struct from [mywheel-rs], adapted to the
/// digraphx-rs code conventions.
///
/// [mywheel-rs]: https://github.com/luk036/mywheel-rs
///
/// # Performance
///
/// | Operation | Complexity |
/// |-----------|------------|
/// | `get`     | O(1)       |
/// | `set`     | O(1)       |
/// | `len`     | O(1)       |
/// | `items`   | O(n)       |
///
/// # Examples
///
/// ```
/// use digraphx_rs::map_adapter::MapAdapter;
///
/// let mut m = MapAdapter::new(vec![10, 20, 30]);
/// assert_eq!(m[0], 10);
/// assert_eq!(m.get(1), Some(&20));
/// assert!(m.contains(2));
/// assert!(!m.contains(5));
///
/// m[1] = 99;
/// assert_eq!(m[1], 99);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapAdapter<T> {
    /// The underlying vector storage.
    pub lst: Vec<T>,
}

impl<T> MapAdapter<T> {
    /// Create a new `MapAdapter` backed by the given vector.
    ///
    /// The adapter exposes the vector's indices as keys in `0..lst.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let m = MapAdapter::new(vec![1, 2, 3]);
    /// assert_eq!(m.len(), 3);
    /// ```
    #[inline]
    pub fn new(lst: Vec<T>) -> Self {
        Self { lst }
    }

    /// Return a reference to the element at `key`, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let m = MapAdapter::new(vec![1, 2, 3]);
    /// assert_eq!(m.get(0), Some(&1));
    /// assert_eq!(m.get(3), None);
    /// ```
    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        self.lst.get(key)
    }

    /// Set the element at `key` to `new_value`.  Does nothing if `key` is out
    /// of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let mut m = MapAdapter::new(vec![1, 2, 3]);
    /// m.set(1, 10);
    /// assert_eq!(m.get(1), Some(&10));
    ///
    /// // Out-of-bounds is a no-op.
    /// m.set(5, 100);
    /// assert_eq!(m.get(5), None);
    /// ```
    #[inline]
    pub fn set(&mut self, key: usize, new_value: T) {
        if let Some(value) = self.lst.get_mut(key) {
            *value = new_value;
        }
    }

    /// Return the number of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// assert_eq!(MapAdapter::new(vec![1, 2, 3]).len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.lst.len()
    }

    /// Return `true` when the adapter contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// assert!(MapAdapter::<i32>::new(vec![]).is_empty());
    /// assert!(!MapAdapter::new(vec![1]).is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    /// Iterate over references to all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let m = MapAdapter::new(vec![1, 2, 3]);
    /// let collected: Vec<&i32> = m.values().collect();
    /// assert_eq!(collected, vec![&1, &2, &3]);
    /// ```
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.lst.iter()
    }

    /// Iterate over `(key, value)` pairs.
    ///
    /// This method mirrors the `.items()` convention found in Python dicts and
    /// the C++ `MapAdapter` in mywheel-cpp, making it straightforward to
    /// implement the [`Graph`](crate::Graph) trait for array-backed containers.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let m = MapAdapter::new(vec![10, 20, 30]);
    /// let pairs: Vec<(usize, &i32)> = m.items().collect();
    /// assert_eq!(pairs, vec![(0, &10), (1, &20), (2, &30)]);
    /// ```
    #[inline]
    pub fn items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.lst.iter().enumerate()
    }

    /// Check whether a given index is in bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use digraphx_rs::map_adapter::MapAdapter;
    ///
    /// let m = MapAdapter::new(vec![1, 2, 3]);
    /// assert!(m.contains(0));
    /// assert!(m.contains(2));
    /// assert!(!m.contains(3));
    /// ```
    #[inline]
    pub fn contains(&self, key: usize) -> bool {
        key < self.lst.len()
    }
}

// --- Index traits -----------------------------------------------------------

impl<T> std::ops::Index<usize> for MapAdapter<T> {
    type Output = T;

    /// # Panics
    ///
    /// Panics if `key` is out of bounds (same as `Vec`).
    #[inline]
    fn index(&self, key: usize) -> &Self::Output {
        &self.lst[key]
    }
}

impl<T> std::ops::IndexMut<usize> for MapAdapter<T> {
    /// # Panics
    ///
    /// Panics if `key` is out of bounds (same as `Vec`).
    #[inline]
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.lst[key]
    }
}

// --- IntoIterator (consuming) ------------------------------------------------

impl<T> IntoIterator for MapAdapter<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.lst.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a MapAdapter<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.lst.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_len() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_get() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        assert_eq!(m.get(0), Some(&1));
        assert_eq!(m.get(2), Some(&3));
        assert_eq!(m.get(3), None);
    }

    #[test]
    fn test_set() {
        let mut m = MapAdapter::new(vec![1, 2, 3]);
        m.set(1, 10);
        assert_eq!(m[1], 10);

        // Out-of-bounds is a no-op
        m.set(10, 999);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        assert!(MapAdapter::<i32>::new(vec![]).is_empty());
        assert!(!MapAdapter::new(vec![1]).is_empty());
    }

    #[test]
    fn test_values() {
        let m = MapAdapter::new(vec![1, 4, 3, 6]);
        assert_eq!(m.values().collect::<Vec<&i32>>(), vec![&1, &4, &3, &6]);
    }

    #[test]
    fn test_items() {
        let m = MapAdapter::new(vec![1, 4, 3, 6]);
        assert_eq!(
            m.items().collect::<Vec<(usize, &i32)>>(),
            vec![(0, &1), (1, &4), (2, &3), (3, &6)]
        );
    }

    #[test]
    fn test_contains() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        assert!(m.contains(0));
        assert!(m.contains(2));
        assert!(!m.contains(3));
    }

    #[test]
    fn test_index() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        assert_eq!(m[0], 1);
        assert_eq!(m[1], 2);
        assert_eq!(m[2], 3);
    }

    #[test]
    fn test_index_mut() {
        let mut m = MapAdapter::new(vec![1, 2, 3]);
        m[1] = 99;
        assert_eq!(m[1], 99);
    }

    #[test]
    fn test_into_iterator() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        let v: Vec<i32> = m.into_iter().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_iterator_ref() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        let v: Vec<&i32> = (&m).into_iter().collect();
        assert_eq!(v, vec![&1, &2, &3]);
    }

    #[test]
    fn test_debug() {
        let m = MapAdapter::new(vec![1, 2, 3]);
        let debug_str = format!("{:?}", m);
        assert!(debug_str.contains("MapAdapter"));
    }

    #[test]
    fn test_clone_eq() {
        let m1 = MapAdapter::new(vec![1, 2, 3]);
        let m2 = m1.clone();
        assert_eq!(m1, m2);
    }
}
