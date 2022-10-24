use std::{collections::HashSet, mem::take, vec};

use crate::{
    sql::{Attribute, FAKE},
    trie::*,
    util::*,
};
use smallvec::SmallVec;

pub fn merge_occurrences(plan: Vec<Instruction2>) -> Vec<Instruction2> {
    let mut groups: Vec<Vec<Instruction2>> = vec![];

    let group_contains = |group: &Vec<Instruction2>, instr: &Instruction2| {
        let (i_left, i_right) = match instr {
            Instruction2::Intersect(inters) => {
                assert_eq!(inters.len(), 2);
                (&inters[0].attr, &inters[1].attr)
            }
            Instruction2::Lookup(lookups) => {
                assert_eq!(lookups.len(), 1);
                (&lookups[0].left, &lookups[0].right)
            }
        };

        group.iter().any(|i| {
            let left;
            let right;
            match i {
                Instruction2::Lookup(lookups) => {
                    assert_eq!(lookups.len(), 1);
                    left = &lookups[0].left;
                    right = &lookups[0].right;
                }
                Instruction2::Intersect(inters) => {
                    assert_eq!(inters.len(), 2);
                    left = &inters[0].attr;
                    right = &inters[1].attr;
                }
            }
            left == i_left || left == i_right || right == i_left || right == i_right
        })
    };

    for instr in plan {
        if let Some(i) = groups.iter().position(|g| group_contains(g, &instr)) {
            groups[i].push(instr);
        } else {
            groups.push(vec![instr]);
        }
    }

    let mut new_plan = vec![];

    for group in groups {
        if group
            .iter()
            .any(|g| matches!(g, Instruction2::Intersect(_)))
        {
            let mut new_inters = vec![];
            for instr in group {
                match instr {
                    Instruction2::Lookup(lookups) => {
                        new_inters.push(Intersection2 {
                            relation: usize::default(),
                            attr: lookups[0].left.clone(),
                        });
                        new_inters.push(Intersection2 {
                            relation: usize::default(),
                            attr: lookups[0].right.clone(),
                        });
                    }
                    Instruction2::Intersect(inters) => {
                        for inter in inters {
                            new_inters.push(inter.clone());
                        }
                    }
                }
            }
            new_plan.push(Instruction2::Intersect(new_inters));
        } else {
            for lookup in group {
                new_plan.push(lookup);
            }
        }
    }

    new_plan
}

pub fn combine_lookups(_optimize: usize, plan: Vec<Instruction2>) -> Vec<Instruction2> {
    // combining lookups is unconditional
    let mut new_plan = vec![];
    let mut current_lookups: Vec<Lookup2> = vec![];

    for instr in plan.clone() {
        if let Instruction2::Lookup(lookups) = instr {
            assert_eq!(lookups.len(), 1);

            let mut not_mergeable = vec![];
            for lookup in lookups {
                if current_lookups
                    .iter()
                    .any(|l| l.relation == lookup.relation)
                {
                    not_mergeable.push(lookup);
                } else {
                    current_lookups.push(lookup);
                }
            }

            if !not_mergeable.is_empty() {
                panic!("not mergeable");
                // assert!(!current_lookups.is_empty());
                // new_plan.push(Instruction2::Lookup(take(&mut current_lookups)));
                // new_plan.push(Instruction2::Lookup(not_mergeable));
            }
        } else {
            if !current_lookups.is_empty() {
                new_plan.push(Instruction2::Lookup(take(&mut current_lookups)));
            }
            new_plan.push(instr);
        }
    }

    if !current_lookups.is_empty() {
        new_plan.push(Instruction2::Lookup(current_lookups));
    }

    for instr in &new_plan {
        if let Instruction2::Lookup(lookups) = instr {
            assert!(!lookups.is_empty());
            let relations: HashSet<usize> = lookups.iter().map(|l| l.relation).collect();
            assert_eq!(lookups.len(), relations.len());
        }
    }

    assert!(new_plan.len() <= plan.len());
    new_plan
}

// pub fn bushy_join<'a>(
//     tables: &[Tb<'a, '_, Value<'a>>],
//     compiled_plan: &[Vec<usize>],
//     tuple: &mut Vec<Value<'a>>,
//     out: &mut Vec<Vec<Value<'a>>>,
// ) {
//     let js = &compiled_plan[0];

//     for j_min in js {
//         if let Tb::Arr((id_cols, data_cols)) = &tables[*j_min] {
//             for i in 0..id_cols[0].len() {
//                 let mut trie_min = Trie::default();
//                 let ids: Vec<_> = id_cols.iter().map(|c| c[i].as_num()).collect();
//                 let data: Vec<_> = data_cols.iter().map(|c| c[i].clone()).collect();
//                 trie_min.insert(&ids, data);
//                 let rels: Vec<_> = tables
//                     .iter()
//                     .map(|t| match &t {
//                         Tb::Arr(_) => &trie_min,
//                         Tb::Trie(trie) => trie,
//                     })
//                     .collect();
//                 bushy_join_inner(&rels, compiled_plan, tuple, out);
//             }
//             return;
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Scan,
    Intersect { relations: Vec<usize> },
    Lookup(Vec<Lookup>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction2 {
    Intersect(Vec<Intersection2>),
    Lookup(Vec<Lookup2>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Intersection2 {
    pub relation: usize,
    pub attr: Attribute,
}

impl std::fmt::Debug for Intersection2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.relation == FAKE {
            write!(f, "{:?}", self.attr)
        } else {
            write!(f, "{:?} @ {:?})", self.attr, self.relation)
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Lookup2 {
    pub key: usize,
    pub relation: usize,
    pub left: Attribute,
    pub right: Attribute,
}

impl std::fmt::Debug for Lookup2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.relation == FAKE {
            write!(f, "{:?} = {:?})", self.left, self.right)
        } else {
            write!(
                f,
                "{:?} @ {:?} = {:?} @ {:?})",
                self.key, self.left, self.relation, self.right
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lookup {
    pub key: usize, // index into the tuple
    pub relation: usize,
}

// Assumes that the first table is a scan
pub fn free_join(
    vectorize: usize,
    optimize: usize,
    tables: &[Table],
    compiled_plan: &[Instruction2],
    out: &mut View,
) {
    let mut compiled_plan = compiled_plan.to_vec();
    println!("n instructions: {}", compiled_plan.len());
    if let Table::Arr { id_cols, data_cols } = &tables[0] {
        let roots: SmallVec<[_; 8]> = tables[1..]
            .iter()
            .map(|t| match &t {
                Table::Arr { .. } => unreachable!("Only left table can be flat"),
                Table::Trie(trie) => trie,
            })
            .collect();

        let rels: SmallVec<[TrieRef; 8]> = roots.iter().map(|t| t.as_ref()).collect::<_>();

        let mut id_iters: Vec<_> = id_cols.iter().map(|c| c.ints().iter().copied()).collect();
        let mut data_iters: Vec<_> = data_cols.iter().map(|c| c.values()).collect();

        // if args.optimize > 0 {
        if optimize == 1 {
            for instr in &mut compiled_plan {
                if let Instruction2::Lookup(lookups) = instr {
                    lookups.sort_unstable_by_key(|l| rels[l.relation].guess_len());
                }
            }
        }

        // assert!(matches!(&compiled_plan[0], Instruction::Scan));

        let mut ctx = JoinContext {
            n_lookups: 0,
            singleton: vec![],
            tuple: vec![],
            out,
        };

        if let [Instruction2::Lookup(lookups), tail @ ..] = &mut compiled_plan[..] {
            let mut local_rels = rels.clone();
            let batch_size = vectorize.min(id_cols[0].len());
            // let batch_size = 1.min(id_cols[0].len()); // ABLATION
            let mut tuple_cols: Vec<Vec<Id>> = vec![Vec::with_capacity(batch_size); id_cols.len()];
            let mut data_cols: Vec<Vec<Value>> =
                vec![Vec::with_capacity(batch_size); data_cols.len()];
            let mut trie_cols = vec![vec![None; batch_size]; lookups.len()];
            'outer: for loop_iter in 0.. {
                for (id_iter, tuple_col) in id_iters.iter_mut().zip(tuple_cols.iter_mut()) {
                    tuple_col.extend(id_iter.take(batch_size));
                    if tuple_col.is_empty() {
                        break 'outer;
                    }
                }

                for (data_iter, data_col) in data_iters.iter_mut().zip(data_cols.iter_mut()) {
                    data_col.extend(data_iter.take(batch_size));
                }

                if loop_iter > 0 {
                    for trie_col in trie_cols.iter_mut() {
                        trie_col.fill(None);
                    }
                }

                // do the first lookup
                let trie_col = &mut trie_cols[0];
                let col = &tuple_cols[lookups[0].key];
                let rel = &rels[lookups[0].relation];
                rel.get_many(
                    col,
                    |_| true,
                    |i, trie| {
                        trie_col[i] = Some(trie);
                    },
                );

                for (j, lookup) in lookups.iter().enumerate().skip(1) {
                    let (tc0, tc1) = trie_cols.split_at_mut(j);
                    let prev_trie_col = tc0.last().unwrap();
                    let trie_col = tc1.first_mut().unwrap();
                    let col = &tuple_cols[lookup.key];
                    let rel = &rels[lookup.relation];
                    rel.get_many(
                        col,
                        |i| prev_trie_col[i].is_some(),
                        |i, trie| {
                            trie_col[i] = Some(trie);
                        },
                    );
                }

                for i in 0..tuple_cols[0].len() {
                    if trie_cols.iter().any(|c| c[i].is_none()) {
                        continue;
                    }

                    for tuple_col in &mut tuple_cols {
                        ctx.tuple.push(tuple_col[i]);
                    }

                    for data_col in &mut data_cols {
                        ctx.singleton.push(data_col[i].clone());
                    }

                    for (lookup, trie_col) in lookups.iter().zip(&mut trie_cols) {
                        local_rels[lookup.relation] = trie_col[i].unwrap();
                    }

                    ctx.join(&local_rels, tail);
                    ctx.tuple.clear();
                    ctx.singleton.clear();
                }

                tuple_cols.iter_mut().for_each(|c| c.clear());
                data_cols.iter_mut().for_each(|c| c.clear());
            }
        } else {
            // unroll the outer scan
            while let Some(v) = id_iters[0].next() {
                ctx.tuple.push(v);

                for id_iter in &mut id_iters[1..] {
                    ctx.tuple.push(id_iter.next().unwrap());
                }

                for data_iter in &mut data_iters {
                    ctx.singleton.push(data_iter.next().unwrap().clone());
                }

                ctx.join(&rels, &mut compiled_plan);
                ctx.tuple.clear();
                ctx.singleton.clear();
            }
        }

        log::debug!("{} lookups", ctx.n_lookups);

        // for i in 0..id_cols[0].len() {
        //     let singleton: SmallVec<[_; 4]> = id_cols
        //         .iter()
        //         .map(|c| &c[i])
        //         .chain(
        //             data_cols
        //                 .iter()
        //                 .map(|c| &c[i])
        //         ).collect();
        //     singleton_join_inner(&singleton,&rels, compiled_plan, tuple, out);
        // }
    } else {
        unreachable!("The first table must be flat");
    }
}

struct JoinContext<'a> {
    n_lookups: usize,
    singleton: Vec<Value>,
    tuple: Vec<Id>,
    out: &'a mut View,
}

impl JoinContext<'_> {
    fn join(&mut self, relations: &[TrieRef], compiled_plan: &mut [Instruction2]) {
        let (instr, rest) = if let Some(tup) = compiled_plan.split_first_mut() {
            tup
        } else {
            return self.materialize(relations);
        };

        match instr {
            Instruction2::Intersect(js) => {
                let j_min = js
                    .iter()
                    .min_by_key(|&j| relations[j.relation].len())
                    .unwrap()
                    .relation;

                // for (&id, trie_min) in relations[j_min].get_map() {
                relations[j_min].for_each(|id, trie_min| {
                    if let Some(tries) = js
                        .iter()
                        .filter(|&j| j.relation != j_min)
                        .map(|j| relations[j.relation].get(id).map(|trie| (j, trie)))
                        .collect::<Option<SmallVec<[_; 8]>>>()
                    {
                        let mut rels: SmallVec<[_; 8]> = SmallVec::from_slice(relations);
                        rels[j_min] = trie_min;
                        for (j, trie) in tries {
                            rels[j.relation] = trie;
                        }
                        self.tuple.push(id);
                        self.join(&rels, rest);
                        self.tuple.pop();
                    }
                });
            }
            Instruction2::Lookup(lookups) => {
                assert!(!lookups.is_empty());
                // if self.args.optimize > 0 {
                //     lookups.sort_unstable_by_key(|l| relations[l.relation].guess_len());
                // }

                let mut rels: SmallVec<[_; 8]> = SmallVec::from_slice(relations);
                for lookup in lookups {
                    let value = self.tuple[lookup.key];
                    self.n_lookups += 1;
                    if let Some(r) = rels[lookup.relation].get(value) {
                        rels[lookup.relation] = r;
                    } else {
                        return;
                    }
                }

                if rest.is_empty() {
                    self.materialize(&rels);
                } else {
                    self.join(&rels, rest);
                }
            }
        }
    }

    fn materialize(&mut self, relations: &[TrieRef]) {
        let out = &mut self.out.vec;
        match relations {
            [] => {
                assert_eq!(self.out.arity, self.tuple.len() + self.singleton.len());
                out.extend(self.tuple.iter().map(|i| Value::Num(*i)));
                out.extend_from_slice(&self.singleton);
            }
            [r] => r.for_each_data(|vs| {
                out.extend(self.tuple.iter().map(|i| Value::Num(*i)));
                out.extend_from_slice(&self.singleton);
                out.extend_from_slice(vs);
            }),
            [r0, r1] => r0.for_each_data(|vs0| {
                r1.for_each_data(|vs1| {
                    out.extend(self.tuple.iter().map(|i| Value::Num(*i)));
                    out.extend_from_slice(&self.singleton);
                    out.extend_from_slice(vs0);
                    out.extend_from_slice(vs1);
                })
            }),
            [r0, r1, r2] => r0.for_each_data(|vs0| {
                r1.for_each_data(|vs1| {
                    r2.for_each_data(|vs2| {
                        out.extend(self.tuple.iter().map(|i| Value::Num(*i)));
                        out.extend_from_slice(&self.singleton);
                        out.extend_from_slice(vs0);
                        out.extend_from_slice(vs1);
                        out.extend_from_slice(vs2);
                    })
                })
            }),
            [r0, r1, r2, r3] => r0.for_each_data(|vs0| {
                r1.for_each_data(|vs1| {
                    r2.for_each_data(|vs2| {
                        r3.for_each_data(|vs3| {
                            out.extend(self.tuple.iter().map(|i| Value::Num(*i)));
                            out.extend_from_slice(&self.singleton);
                            out.extend_from_slice(vs0);
                            out.extend_from_slice(vs1);
                            out.extend_from_slice(vs2);
                            out.extend_from_slice(vs3);
                        })
                    })
                })
            }),
            _ => relations[0].for_each_data(|vs| {
                let len = self.singleton.len();
                self.singleton.extend_from_slice(vs);
                self.materialize(&relations[1..]);
                self.singleton.truncate(len);
            }),
        }
    }
}

// fn materialize_single<'a>(
//     relations: &[&Trie<Value<'a>>],
//     tuple: &mut Vec<Value<'a>>,
//     out: &mut Vec<Vec<Value<'a>>>,
// ) {
//     if relations.is_empty() {
//         out.push(tuple.to_vec());
//     } else if relations[0].get_data().unwrap().is_empty() {
//         materialize_single(singleton, &relations[1..], tuple, out);
//     } else {
//         for vs in relations[0].get_data().unwrap() {
//             for v in vs {
//                 tuple.push(v.clone());
//             }
//             materialize_single(singleton, &relations[1..], tuple, out);
//             for _v in vs {
//                 tuple.pop();
//             }
//         }
//     }
// }

// fn bushy_join_inner<'a>(
//     relations: &[&Trie<Value<'a>>],
//     compiled_plan: &[Vec<usize>],
//     tuple: &mut Vec<Value<'a>>,
//     out: &mut Vec<Vec<Value<'a>>>,
// ) {
//     if compiled_plan.is_empty() {
//         materialize(relations, tuple, out);
//     } else {
//         let js = &compiled_plan[0];

//         let j_min = js
//             .iter()
//             .copied()
//             .min_by_key(|&j| relations[j].get_map().unwrap().len())
//             .unwrap();

//         for (id, trie_min) in relations[j_min].get_map().unwrap().iter() {
//             if let Some(tries) = js
//                 .iter()
//                 .filter(|&j| j != &j_min)
//                 .map(|&j| {
//                     relations[j]
//                         .get_map()
//                         .unwrap()
//                         .get(id)
//                         .map(|trie| (j, trie))
//                 })
//                 .collect::<Option<Vec<_>>>()
//             {
//                 let mut rels = relations.to_vec();
//                 rels[j_min] = trie_min;
//                 for (j, trie) in tries {
//                     rels[j] = trie;
//                 }
//                 tuple.push(Value::Num(*id));
//                 bushy_join_inner(&rels, &compiled_plan[1..], tuple, out);
//                 tuple.pop();
//             }
//         }
//     }
// }
