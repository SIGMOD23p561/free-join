use std::convert::TryInto;
use std::fmt::Debug;
use std::rc::Rc;

use hashbrown::hash_map::Entry;
use smallvec::{smallvec, SmallVec};

use crate::sql::Attribute;
use crate::*;

#[inline]
#[cold]
fn cold() {}

mod cell {
    use super::cold;
    use std::cell::UnsafeCell;

    pub struct OnceCell<T, Z> {
        inner: UnsafeCell<Result<T, Z>>,
    }

    impl<T, Z> OnceCell<T, Z> {
        /// Creates a new empty cell.
        pub const fn new(init: Z) -> OnceCell<T, Z> {
            OnceCell {
                inner: UnsafeCell::new(Err(init)),
            }
        }

        pub fn get(&self) -> Option<&T> {
            match unsafe { &*self.inner.get() } {
                Ok(v) => Some(v),
                Err(_) => {
                    cold();
                    None
                }
            }
        }

        pub fn get_init_data(&mut self) -> &mut Z {
            match self.inner.get_mut() {
                Ok(_) => unsafe { std::hint::unreachable_unchecked() },
                Err(z) => z,
            }
        }

        pub fn map<Out, FT, FZ>(&self, ft: FT, fz: FZ) -> Out
        where
            FT: FnOnce(&T) -> Out,
            FZ: FnOnce(&Z) -> Out,
        {
            match unsafe { &*self.inner.get() } {
                Ok(v) => ft(v),
                Err(z) => fz(z),
            }
        }

        pub fn get_or_init<F>(&self, f: F) -> &T
        where
            F: FnOnce(Z) -> T,
        {
            if let Some(v) = self.get() {
                return v;
            }
            cold();
            let z = match unsafe { &mut *self.inner.get() } {
                Ok(_) => unsafe { std::hint::unreachable_unchecked() },
                Err(z) => unsafe { std::ptr::read(z) },
            };
            let v = f(z);
            unsafe { std::ptr::write(self.inner.get(), Ok(v)) };
            self.get().unwrap()
        }
    }
}

pub type Id = i32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Value {
    Str(Rc<String>),
    Num(Id),
}

impl Value {
    pub fn as_num(&self) -> Id {
        match self {
            Value::Num(n) => *n,
            Value::Str(_) => panic!("Value is not a number"),
        }
    }
}

struct Trie {
    inner: cell::OnceCell<Box<TrieInner>, Thunk>,
}

pub struct TrieRoot {
    schema: Box<TrieSchema>,
    trie: Trie,
}

#[derive(Clone, Copy)]
pub struct TrieRef<'a> {
    schema: &'a TrieSchema,
    trie: &'a Trie,
}

struct Thunk(SmallVec<[u32; 4]>);

impl Thunk {
    fn empty() -> Self {
        Self(Default::default())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn singleton(idx: u32) -> Thunk {
        Self(smallvec![idx])
    }

    fn push(&mut self, idx: u32) {
        self.0.push(idx)
    }

    #[inline(always)]
    fn for_each(&self, mut f: impl FnMut(u32)) {
        for &i in &self.0 {
            f(i)
        }
    }
}

type InnerMap = HashMap<Id, u32>;

enum TrieInner {
    Map(InnerMap, Vec<Trie>),
    Set(HashSet<Id>, Box<Trie>),
    DenseSet(DenseMap<()>, Box<Trie>),
    Data(Thunk),
}

struct DenseMap<T> {
    min: Id,
    map: Vec<Option<T>>,
}

impl<T: Copy> DenseMap<T> {
    fn new(min: Id, max: Id) -> Self {
        let len: usize = (max - min + 1).try_into().unwrap();
        Self {
            map: vec![None; len],
            min,
        }
    }

    fn insert(&mut self, id: Id, value: T) {
        let i: usize = (id - self.min).try_into().unwrap();
        self.map[i] = Some(value);
    }

    fn get(&self, id: Id) -> Option<T> {
        let i: usize = (id - self.min).try_into().ok()?;
        self.map.get(i).copied().flatten()
    }

    fn iter(&self) -> impl Iterator<Item = (Id, T)> + '_ {
        self.map
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|v| (i as Id + self.min, v)))
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

pub enum TrieSchema {
    Id(Col, Box<Self>),
    Data(Vec<Col>),
}

impl TrieSchema {
    pub fn new(id_cols: Vec<Col>, data_cols: Vec<Col>) -> Self {
        let mut schema = Self::Data(data_cols.to_vec());
        for col in id_cols.into_iter().rev() {
            schema = Self::Id(col, Box::new(schema));
        }
        schema
    }

    fn next(&self) -> &Self {
        match self {
            Self::Id(_, next) => next,
            Self::Data(_) => panic!("No next schema"),
        }
    }

    fn id_col(&self) -> &[Id] {
        match self {
            TrieSchema::Id(col, _) => col.ints(),
            TrieSchema::Data(_) => panic!("not an id schema"),
        }
    }

    fn data_cols(&self) -> &[Col] {
        match self {
            TrieSchema::Id(..) => panic!("not a data schema"),
            TrieSchema::Data(cols) => cols,
        }
    }
}

pub type Schema = Vec<Attribute>;

pub enum Table {
    Trie(TrieRoot),
    Arr {
        id_cols: Vec<Col>,
        data_cols: Vec<Col>,
    },
}

impl TrieRoot {
    pub fn new(schema: TrieSchema) -> Self {
        Self {
            schema: Box::new(schema),
            trie: Trie {
                inner: cell::OnceCell::new(Thunk::empty()),
            },
        }
    }

    pub fn as_ref(&self) -> TrieRef {
        TrieRef {
            schema: &self.schema,
            trie: &self.trie,
        }
    }
}

impl<'a> TrieRef<'a> {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        match self.force() {
            TrieInner::Map(map, _) => map.len(),
            TrieInner::Set(set, _t) => set.len(),
            TrieInner::DenseSet(set, _t) => set.len(),
            TrieInner::Data(..) => panic!(),
        }
    }

    pub fn guess_len(self) -> usize {
        self.trie.inner.map(
            |trie| match trie.as_ref() {
                TrieInner::Map(m, _) => m.len(),
                TrieInner::Set(s, _) => s.len(),
                TrieInner::DenseSet(s, _) => s.len(),
                TrieInner::Data(..) => panic!(),
            },
            |thunk| match thunk.len() {
                0 => self.schema.id_col().len(),
                _ => thunk.len(),
            },
        )
    }

    #[cold]
    #[inline(never)]
    fn make_trie(self, thunk: Thunk) -> Box<TrieInner> {
        let kind = match self.schema {
            TrieSchema::Data(_) => 0,
            TrieSchema::Id(_, next) => match next.as_ref() {
                TrieSchema::Data(data_cols) if data_cols.is_empty() => 1,
                _ => 2,
            },
        };

        // println!("size of trie: {}", std::mem::size_of::<Trie>());

        let mk_thunk = |i: u32| Trie {
            inner: cell::OnceCell::new(Thunk::singleton(i)),
        };

        let inner = if kind == 0 {
            TrieInner::Data(thunk)
        } else if kind == 1 {
            let mut set = HashSet::<Id>::default();
            let mut min = Id::MAX;
            let mut max = Id::MIN;
            self.for_each_in_thunk(thunk, |_i, id| {
                min = min.min(id);
                max = max.max(id);
                set.insert(id);
            });

            assert!(!set.is_empty());
            // println!("Trie len: {}", set.len());

            if (max - min) < ((set.len() * 2) as Id) {
                let mut denseset = DenseMap::new(min, max);
                for id in set {
                    denseset.insert(id, ());
                }
                TrieInner::DenseSet(denseset, Box::new(mk_thunk(0xc0ffee)))
            } else {
                TrieInner::Set(set, Box::new(mk_thunk(0xc0ffee)))
            }
        } else {
            let mut map = InnerMap::default();
            let mut tries = Vec::new();

            self.for_each_in_thunk(thunk, |i, id| match map.entry(id) {
                Entry::Vacant(e) => {
                    e.insert(tries.len() as _);
                    tries.push(mk_thunk(i));
                }
                Entry::Occupied(e) => {
                    let trie_idx = *e.get() as usize;
                    tries[trie_idx].inner.get_init_data().push(i);
                }
            });

            // println!("Trie len: {}", map.len());

            TrieInner::Map(map, tries)
        };

        Box::new(inner)
    }

    fn for_each_in_thunk(self, thunk: Thunk, mut f: impl FnMut(u32, Id)) {
        let col = self.schema.id_col();
        if thunk.is_empty() {
            for (i, &id) in col.iter().enumerate() {
                f(i as u32, id);
            }
        } else {
            thunk.for_each(|idx| {
                f(idx, col[idx as usize]);
            });
        }
    }

    #[inline(always)]
    fn force(self) -> &'a TrieInner {
        self.trie.inner.get_or_init(|thunk| self.make_trie(thunk))
    }

    pub fn force_one(self) {
        self.force();
    }

    pub fn force_all(self) {
        match self.force() {
            TrieInner::Data(_) | TrieInner::DenseSet(..) | TrieInner::Set(..) => {}
            TrieInner::Map(..) => self.for_each(|_, t| t.force_all()),
        }
    }

    fn mk_ref(self, trie: &'a Trie) -> TrieRef<'a> {
        TrieRef {
            schema: self.schema.next(),
            trie,
        }
    }

    pub fn get(self, id: Id) -> Option<TrieRef<'a>> {
        match self.force() {
            TrieInner::Map(map, ts) => map.get(&id).map(|&i| self.mk_ref(&ts[i as usize])),
            TrieInner::Set(set, trie) => {
                if set.contains(&id) {
                    Some(self.mk_ref(trie))
                } else {
                    None
                }
            }
            TrieInner::DenseSet(s, trie) => {
                if s.get(id).is_some() {
                    Some(self.mk_ref(trie))
                } else {
                    None
                }
            }
            TrieInner::Data(..) => panic!("Trie is not a node"),
        }
    }

    pub fn get_many<F1, F2>(self, ids: &[Id], test: F1, mut f: F2)
    where
        F1: Fn(usize) -> bool,
        F2: FnMut(usize, Self),
    {
        match self.force() {
            TrieInner::Map(map, ts) => {
                let schema = self.schema.next();
                for (i, &id) in ids.iter().enumerate() {
                    if test(i) {
                        if let Some(ti) = map.get(&id) {
                            let trie = &ts[*ti as usize];
                            f(i, Self { schema, trie });
                        }
                    }
                }
            }
            TrieInner::Set(set, trie) => {
                let schema = self.schema.next();
                for (i, &id) in ids.iter().enumerate() {
                    if test(i) && set.contains(&id) {
                        f(i, Self { schema, trie });
                    }
                }
            }
            TrieInner::DenseSet(set, trie) => {
                let schema = self.schema.next();
                for (i, &id) in ids.iter().enumerate() {
                    if test(i) && set.get(id).is_some() {
                        f(i, Self { schema, trie });
                    }
                }
            }
            TrieInner::Data(..) => panic!("Trie is not a node"),
        }
    }

    pub fn for_each(self, mut f: impl FnMut(Id, Self)) {
        match self.force() {
            TrieInner::Map(map, ts) => {
                let schema = self.schema.next();
                for (&id, &ti) in map {
                    let trie = &ts[ti as usize];
                    f(id, Self { schema, trie });
                }
            }
            TrieInner::Set(set, trie) => {
                let schema = self.schema.next();
                for &id in set {
                    f(id, Self { schema, trie });
                }
            }
            TrieInner::DenseSet(set, trie) => {
                let schema = self.schema.next();
                for (id, _) in set.iter() {
                    f(id, Self { schema, trie });
                }
            }
            TrieInner::Data(..) => panic!("Trie is not a node"),
        }
    }

    pub fn has_data(self) -> bool {
        match self.force() {
            TrieInner::Set(..) => false,
            TrieInner::Map(..) => panic!(),
            TrieInner::DenseSet(..) => panic!(),
            TrieInner::Data(..) => !self.schema.data_cols().is_empty(),
        }
    }

    pub fn for_each_data(self, mut f: impl FnMut(&[Value])) {
        match self.force() {
            TrieInner::Map(..) => panic!(),
            TrieInner::Set(..) => panic!(),
            TrieInner::DenseSet(..) => panic!(),
            TrieInner::Data(thunk) => {
                if thunk.is_empty() {
                    f(&[])
                } else {
                    let mut row = SmallVec::<[Value; 4]>::default();
                    thunk.for_each(|idx| {
                        for col in self.schema.data_cols() {
                            row.push(col.get_value(idx as usize).clone());
                        }
                        f(&row);
                        row.clear();
                    })
                }
            }
        }
    }
}
