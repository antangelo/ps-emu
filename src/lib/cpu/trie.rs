use std::rc::Rc;

type TrieNodeEntry<T> = Vec<Option<Rc<T>>>;

#[derive(Debug)]
pub struct Trie<T> {
    l1: Vec<Option<TrieNodeEntry<T>>>,
}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self {
            l1: vec![None; 1 << (32 - 8)],
        }
    }
}

impl<T> Trie<T> {
    pub fn insert(&mut self, addr: u32, data: Rc<T>) -> Result<(), String> {
        let hi = addr >> 8;
        let lo = (addr >> 2) & 0x3f;

        let l1 = &mut self.l1[hi as usize];
        if l1.is_none() {
            *l1 = Some(vec![None; 1 << 6]);
        }

        l1.as_mut().unwrap()[lo as usize] = Some(data.clone());
        Ok(())
    }

    pub fn invalidate(&mut self, addr: u32) {
        let hi = addr >> 8;
        self.l1[hi as usize] = None;
    }

    pub fn lookup(&self, addr: u32) -> Option<std::rc::Rc<T>> {
        let hi = addr >> 8;
        let lo = (addr >> 2) & 0x3f;

        let l1 = self.l1[hi as usize].as_ref()?;
        let l2 = l1[lo as usize].as_ref()?.clone();
        Some(l2)
    }
}

#[cfg(test)]
mod test {
    use super::Trie;
    use std::rc::Rc;

    #[test]
    fn trie_test_insert_contains_fetch() {
        let mut trie: Trie<u32> = Trie::default();

        trie.insert(0x1000, Rc::new(10)).unwrap();
        trie.insert(0x2000, Rc::new(15)).unwrap();

        assert_eq!(&10, trie.lookup(0x1000).unwrap().as_ref());
    }
}
