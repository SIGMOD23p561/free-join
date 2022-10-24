use itertools::Itertools;
use std::cmp;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use translator::*;
use walkdir::WalkDir;
use translator::JoinType::Inner;

// 0. Each Join is hash join
fn check_each_join_is_hash_join(root: &TreeOp) -> bool {
    let map_func = |node: &TreeOp| -> bool {
        // We only care about the join without hash impl that it should be false;
        !matches!(node.name.as_str(), str if str.contains("JOIN") && !str.contains("HASH"))
    };
    let reduce_func = |a: bool, b: bool| -> bool { a && b };
    let combine_func = |a: bool, b: bool| -> bool { a && b };
    let default_func = || true;
    let traverse_funcs = TraverseFuncs {
        map_func: &map_func,
        combine_func: &combine_func,
        reduce_func: &reduce_func,
        default_func: &default_func,
    };
    traverse(root, &traverse_funcs)
}

// 1. Bushy Join is bushy or not the width of join trees BFS{}
fn check_only_contain_linear_join(root: &TreeOp) -> bool {
    let map_func = |node: &TreeOp| -> usize {
        match &node.attr {
            Some(NodeAttr::Join(join_attr)) => {
                if join_attr.join_type == Inner {
                    node.children.len()
                } else {
                    0
                }
            },
            _ => 0,
        }
    };
    let reduce_func = |a: usize, b: usize| -> usize { a + b };
    let combine_func = |map: usize, reduce: usize| -> usize { cmp::max(map, reduce) };
    let default_func = || 0;
    let traverse_funcs = TraverseFuncs {
        map_func: &map_func,
        combine_func: &combine_func,
        reduce_func: &reduce_func,
        default_func: &default_func,
    };
    traverse(root, &traverse_funcs) == 2
}
// 2. Aggregates is only on the top

fn check_aggregate_is_on_top_only(root: &TreeOp) -> bool {
    struct TraverseState {
        is_aggr: bool,
        is_join: bool,
        results: bool,
    }

    let map_func = |node: &TreeOp| -> TraverseState {
        match node {
            n if n.name.contains("AGGREGATE") => TraverseState {
                is_aggr: true,
                is_join: false,
                results: true,
            },
            n if n.name.contains("JOIN") &&
                matches!(&n.attr, Some(NodeAttr::Join(join_attr)) if join_attr.join_type == Inner)
            => TraverseState {
                is_aggr: false,
                is_join: true,
                results: true,
            },
            _ => TraverseState {
                is_aggr: false,
                is_join: false,
                results: true,
            },
        }
    };

    let reduce_func = |a: TraverseState, b: TraverseState| TraverseState {
        is_aggr: a.is_aggr || b.is_aggr, // ANY recursive Children has aggregate
        is_join: false,                  // Not Necessary since only want to know current nodes
        results: a.results && b.results, // ALL CASES DO NOT CONTAIN AGGR INSIDE JOIN
    };

    let combine_func = |map: TraverseState, reduce: TraverseState| {
        TraverseState {
            is_aggr: map.is_aggr || reduce.is_aggr,
            is_join: false,
            // if current node is join and chileren has aggr, then results should be false
            results: !(map.is_join && reduce.is_aggr),
        }
    };

    let default_func = || TraverseState {
        is_aggr: false,
        is_join: false,
        results: true,
    };

    let traverse_funcs = TraverseFuncs {
        map_func: &map_func,
        combine_func: &combine_func,
        reduce_func: &reduce_func,
        default_func: &default_func,
    };
    let res: TraverseState = traverse(root, &traverse_funcs);
    res.results
}

// 3. Look if any two or more variables share the same table name?
//    We can do the check on GJ Plan
//
fn check_var_has_unique_table_combs(gj_plan: &[Vec<String>]) -> bool {
    let table_set_for_each_var: Vec<HashSet<String>> = gj_plan
        .iter()
        .map(|vs| {
            // Build Unique Hash Set for Table Name of Each Variable
            HashSet::from_iter(vs.iter().map(|s| s.split('.').next().unwrap().to_string()))
        })
        .collect_vec();
    table_set_for_each_var
        .iter()
        .combinations(2)
        .all(|vp| !(vp.first().unwrap().eq(vp.last().unwrap())))
    // Check whether all tables unique set are different
}

// 4. Sort-Merge Join | Sort Trie | Segmented Array is in further project

fn main() {
    let args: Vec<String> = env::args().collect();

    // The input would be directory
    // and traverse all json
    // print name and check
    // check three or four cases and output to the screen



    let mut results: Vec<(String, bool, bool, bool, bool)> = vec![];

    let dirname = &args[1];
    for file in WalkDir::new(dirname)
        .into_iter()
        .filter_map(|file| file.ok())
    {
        if file.metadata().unwrap().is_file()
            && file.path().extension().and_then(OsStr::to_str) == Some("json")
        {
            // File to String
            let contents = fs::read_to_string(file.path())
                .unwrap_or_else(|_| panic!("Cannot read file {}", file.path().display()));

            // Parse the string of data into serde_json::Value.
            let mut root: TreeOp =
                serde_json::from_str(contents.as_str()).expect("Failed to Parse Json!");

            let filename = file.path().file_name().and_then(OsStr::to_str).unwrap().to_string();

            parse_tree_extra_info(&mut root, filename.as_str());
            let allhash = check_each_join_is_hash_join(&root);
            let linear = check_only_contain_linear_join(&root);
            let topaggr = check_aggregate_is_on_top_only(&root);
            // [["t.id", "miidx.movie_id", "mi.movie_id", "mc.movie_id"], ["t.kind_id", "kt.id"], ["mi.info_type_id", "it2.id"], ["miidx.info_type_id", "it.id"], ["mc.company_type_id", "ct.id"], ["mc.company_id", "cn.id"]]
            let gj_plan = to_gj_plan(&mut root);
            let uniqtbl = check_var_has_unique_table_combs(&gj_plan);



            results.push((filename,
                                   allhash,
                                   linear,
                                   topaggr,
                                   uniqtbl));

            // println!("{:?}", gj_plan);
        }

    }

    results.sort_by(|x, y| { x.cmp(y) });

    println!("filename allhash linear topaggr uniqtbl");

    for (filename, allhash, linear, topaggr, uniqtbl) in results.iter()
    {
        println!("{} {:?} {:?} {:?} {:?}", filename, allhash, linear, topaggr, uniqtbl)
    }
}

// Waive
