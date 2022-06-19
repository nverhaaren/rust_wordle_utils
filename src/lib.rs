#[cfg(test)]
mod tests {
    use crate::Clue;
    use crate::checkers::{Solution, check_once};
    #[test]
    fn basic_check() {
        let mut solution = Solution::new(String::from("anise"));
        assert_eq!(solution.check(&String::from("shine")).collect::<Vec<_>>(), vec![
            Clue::Present, Clue::Absent, Clue::Exact, Clue::Present, Clue::Exact
        ]);
    }
    #[test]
    fn multi_check() {
        let mut solution = Solution::new(String::from("ender"));
        assert_eq!(solution.check(&String::from("peeve")).collect::<Vec<_>>(), vec![
            Clue::Absent, Clue::Present, Clue::Present, Clue::Absent, Clue::Absent
        ]);
        assert_eq!(solution.check(&String::from("bevel")).collect::<Vec<_>>(), vec![
            Clue::Absent, Clue::Present, Clue::Absent, Clue::Exact, Clue::Absent
        ]);
    }
    #[test]
    fn once_check() {
        assert_eq!(
            check_once("ender", "peeve").collect::<Vec<_>>(),
            vec![Clue::Absent, Clue::Present, Clue::Present, Clue::Absent, Clue::Absent]
        );
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Clue {
    Exact,
    Present,
    Absent,
}


// Unused right now, seems like it would work but in the case below would require multiple
// mutable references. Still could be useful as an example.
#[allow(dead_code)]
mod drop_iter {
    use std::iter::Iterator;
    pub struct DropIterator<I: Iterator, F: FnMut()> {
        underlying: I,
        drop: F,
    }

    impl<I: Iterator, F: FnMut()> DropIterator<I, F> {
        fn new(underlying: I, drop: F) -> DropIterator<I, F> {
            DropIterator { underlying, drop }
        }
    }

    impl<I, F> Iterator for DropIterator<I, F>
    where I: Iterator, F: FnMut() {
        type Item = I::Item;
        fn next(&mut self) -> Option<Self::Item> {
            self.underlying.next()
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.underlying.size_hint()
        }
    }

    impl<I, F> Drop for DropIterator<I, F>
    where I: Iterator, F: FnMut() {
        fn drop(&mut self) {
            (self.drop)()
        }
    }

    pub trait HasDropIterator : Iterator {
        fn drop_iterator<F: FnMut()>(self, drop: F) -> DropIterator<Self, F>
        where Self: Sized {
            DropIterator::new(self, drop)
        }
    }

    impl<I: Iterator> HasDropIterator for I {}
}

mod drop_map {
    pub struct DropMap<I: Iterator, C, B, F, D>
    where F: FnMut(&mut C, <I as Iterator>::Item) -> B, D: FnMut(&mut C) {
        iter: I,
        context: C,
        map_fn: F,
        drop_fn: D,
    }

    // impl<I, C, B, F, D> DropMap<I, C, F, D>
    // where I: Iterator, F: FnMut(&mut C, <I as Iterator>::Item) -> B, D: FnMut(&mut C) {
    //     pub fn new(iter: I, context: C, map_fn: F, drop_fn: D) -> DropMap<I, C, F, D> {
    //         DropMap { iter, context, map_fn, drop_fn }
    //     }
    // }

    impl<I, C, B, F, D> Iterator for DropMap<I, C, B, F, D>
    where I: Iterator, F: FnMut(&mut C, <I as Iterator>::Item) -> B, D: FnMut(&mut C) {
        type Item = B;
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(|x| (self.map_fn)(&mut self.context, x))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }
    }

    impl<I: Iterator, C, B, F, D> Drop for DropMap<I, C, B, F, D>
    where F: FnMut(&mut C, <I as Iterator>::Item) -> B, D: FnMut(&mut C) {
        fn drop(&mut self) {
            (self.drop_fn)(&mut self.context)
        }
    }

    pub trait HasDropMap : Iterator {
        fn drop_map<C, B, F, D>(self, context: C, map_fn: F, drop_fn: D) -> DropMap<Self, C, B, F, D>
        where Self: Sized, F: FnMut(&mut C, <Self as Iterator>::Item) -> B, D: FnMut(&mut C) {
            DropMap {iter: self, context, map_fn, drop_fn}
        }
    }

    impl<I: Iterator> HasDropMap for I {}
}


pub mod checkers {
    use crate::Clue;
    use std::collections::HashMap;
    use std::iter::Iterator;
    use itertools::Itertools;
    use crate::drop_map::HasDropMap;

    trait ResettableMap<K, V> : AsMut<Self> where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn get_mut(&mut self) -> &mut HashMap<K, V>;
        fn reset(&mut self);
    }

    struct ResetToEmpty<K, V> {
        map: HashMap<K, V>,
    }

    impl<K, V> ResetToEmpty<K, V> where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn new() -> Self {
            Self { map: HashMap::new() }
        }
    }

    // This would be a good place to learn about macros.
    impl<K, V> AsMut<Self> for ResetToEmpty<K, V> where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn as_mut(&mut self) -> &mut Self {
            self
        }
    }

    impl<K, V> ResettableMap<K, V> for ResetToEmpty<K, V>
    where K: Eq + std::hash::Hash + Clone, V: Clone {
        fn get_mut(&mut self) -> &mut HashMap<K, V> {
            &mut self.map
        }
        fn reset(&mut self) {
            self.map.clear();
        }
    }

    struct ResetToOriginal<K, V> {
        original: HashMap<K, V>,
        map: HashMap<K, V>,
    }

    impl<K, V> ResetToOriginal<K, V> where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn new(original: HashMap<K, V>) -> Self {
            let map = original.clone();
            Self { original, map }
        }
    }

    impl<K, V> ResettableMap<K, V> for ResetToOriginal<K, V>
    where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn get_mut(&mut self) -> &mut HashMap<K, V> {
            &mut self.map
        }
        fn reset(&mut self) {
            assert_eq!(
                self.map.len(), self.original.len(), "Only changes to counts allowed on map");
            for (k, v) in self.original.iter() {
                *self.map.get_mut(k).expect(
                    "Only changes to counts allowed on map") = v.clone();
            };
        }
    }

    impl<K, V> AsMut<Self> for ResetToOriginal<K, V> where K: Eq + Clone + std::hash::Hash, V: Clone {
        fn as_mut(&mut self) -> &mut Self {
            self
        }
    }

    fn check<'a, T: ResettableMap<char, u8>>(
        solution: &'a str, guess: &'a str, resettable_counts: impl AsMut<T> + 'a
    ) -> impl Iterator<Item = Clue> + 'a {
        solution.chars().zip_eq(guess.chars()).drop_map(
            resettable_counts,
            |rcounts, (sch, gch)| {
                let counts = rcounts.as_mut().get_mut();
                if sch == gch {
                    Clue::Exact
                } else {
                    if let Some(count) = counts.get_mut(&gch){
                        if *count > 0 {
                            *count -= 1;
                            return Clue::Present;
                        }
                    }
                    return Clue::Absent;
                }
            },
            |rcounts| {
                rcounts.as_mut().reset();
            }
        )
    }

    fn init_counts(counts: &mut HashMap<char, u8>, solution: &str) {
        for ch in solution.chars() {
            let count = counts.entry(ch).or_default();
            *count += 1;
        }
    }

    pub fn check_once<'a>(solution: &'a str, guess: &'a str) -> impl Iterator<Item = Clue> + 'a {
        let mut resettable_counts = ResetToEmpty{map: HashMap::new()};
        init_counts(&mut resettable_counts.get_mut(), solution);
        check(solution, guess, resettable_counts)
    }

    pub struct Solution {
        solution: String,
        counts: ResetToOriginal<char, u8>,
    }

    impl Solution {
        pub fn new(solution: String) -> Solution {
            let mut counts = HashMap::new();
            init_counts(&mut counts, &solution);
            Solution { solution, counts: ResetToOriginal::new(counts) }
        }

        pub fn check<'a>(&'a mut self, guess: &'a str) -> impl Iterator<Item = Clue> + 'a {
            check(&self.solution, &guess, &mut self.counts)
        }
    }

    pub struct Guess {
        guess: String,
        counts: ResetToEmpty<char, u8>,
    }

    impl Guess {
        pub fn new(guess: String) -> Guess {
            Guess { guess, counts: ResetToEmpty::new() }
        }

        pub fn check<'a>(&'a mut self, solution: &'a str) -> impl Iterator<Item = Clue> + 'a {
            init_counts(self.counts.get_mut(), solution);
            check(&solution, &self.guess, &mut self.counts)
        }
    }
}
