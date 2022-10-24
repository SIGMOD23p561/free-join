use serde::Serialize;
use std::time::{Duration, Instant};

use gj::{join::*, sql::*, util::*, *};

use clap::Parser;

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut json = std::fs::File::create(&args.json).unwrap();

    let mut queries = queries();
    if let Some(q) = &args.query {
        queries.retain(|_, name| name.contains(q));
    }

    let mut records = Vec::new();
    let mut ddb_records = Vec::new();

    for (q, _i) in queries {
        println!("running query {} ", q);

        let scan_tree = get_join_tree(&format!("../logs/scan-profiles/{}.json", q)).unwrap();
        let plan_tree = get_join_tree(&format!("../logs/plan-profiles/{}.json", q)).unwrap();

        let total_time = plan_tree.timing;
        let filter_time = get_filter_cost(&plan_tree);
        let time = total_time - filter_time;

        println!("DUCKDB total time: {}", total_time);
        println!("DUCKDB filter time: {}", filter_time);
        println!("DUCKDB join time: {}", time);

        ddb_records.push(DuckDbRecord {
            query: q.into(),
            time,
        });

        let scan = get_scans(&scan_tree);
        let payload = get_payload(&plan_tree);
        let plan = to_gj_plan(&plan_tree);

        let db = load_db(&args, q, &scan, &plan);

        // for optimize in [0, 1, 2] {
        //     records.push(Record {
        //         query: q.into(),
        //         vectorize: 1000,
        //         optimize,
        //         strategy: 2,
        //         time: (0..args.n_trials)
        //             .map(|_| {
        //                 run_query(
        //                     &plan_tree,
        //                     1000,
        //                     optimize,
        //                     BuildStrategy::COLT,
        //                     &db,
        //                     &payload,
        //                 )
        //             })
        //             .collect(),
        //     });
        // }

        // for (i, strategy) in [BuildStrategy::Full, BuildStrategy::SLT, BuildStrategy::COLT]
        //     .into_iter()
        //     .enumerate()
        // {
        //     records.push(Record {
        //         query: q.into(),
        //         vectorize: 1000,
        //         optimize: 1,
        //         strategy: i,
        //         time: (0..args.n_trials)
        //             .map(|_| run_query(&plan_tree, 1000, 1, strategy, &db, &payload))
        //             .collect(),
        //     });
        // }

        // for vectorize in [1, 10, 100, 1000] {
        //     records.push(Record {
        //         query: q.into(),
        //         vectorize,
        //         optimize: 1,
        //         strategy: 2,
        //         time: (0..args.n_trials)
        //             .map(|_| {
        //                 run_query(&plan_tree, vectorize, 1, BuildStrategy::COLT, &db, &payload)
        //             })
        //             .collect(),
        //     });
        // }

        records.push(Record {
            query: q.into(),
            vectorize: 1000,
            optimize: 1,
            strategy: 2,
            time: (0..1)
                .map(|_| run_query(&plan_tree, 1000, 1, BuildStrategy::COLT, &db, &payload))
                .collect(),
        });

        // records.push(Record {
        //     query: q.into(),
        //     vectorize: 1,
        //     optimize: 0,
        //     strategy: 0,
        //     time: (0..args.n_trials)
        //         .map(|_| run_query(&plan_tree, 1, 0, BuildStrategy::Full, &db, &payload))
        //         .collect(),
        // });
    }

    serde_json::to_writer_pretty(
        &mut json,
        &serde_json::json!({
            "gj": records,
            "duckdb": ddb_records,
        }),
    )
    .unwrap();
}

#[derive(Serialize)]
struct Record {
    query: String,
    vectorize: usize,
    optimize: usize,
    strategy: usize,
    time: Vec<f64>,
}

#[derive(Serialize)]
struct DuckDbRecord {
    query: String,
    time: f64,
}

fn run_query(
    plan_tree: &TreeOp,
    vectorize: usize,
    optimize: usize,
    build_strategy: BuildStrategy,
    db: &DB,
    payload: &[&Attribute],
) -> f64 {
    let mut in_view = HashMap::default();
    let mut provides = IndexMap::default();
    let mut build_plans = IndexMap::default();
    let mut compiled_plans = IndexMap::default();
    let tm = to_materialize(plan_tree);
    let root = tm[tm.len() - 1];
    for node in to_materialize(plan_tree) {
        // let groups = to_left_deep_plan(node);
        let mut plan = to_binary_plan2(node, &in_view, &provides);
        // let compiled_plan = compile_plan(&groups, node, &in_view);
        log::debug!("binary plan: {:?}", plan);

        if optimize == 2 {
            plan = merge_occurrences(plan);
            log::debug!("merged plan: {:?}", &plan);
        }

        let (out_schema, build_plan) = compute_full_plan(db, node, &mut plan, &provides, &in_view);
        log::debug!("out schema: {:?}", out_schema);

        plan = combine_lookups(optimize, plan); // TODO disabled for cyclic queries
                                                // compute_full_plan(&db, &groups, &provides, &in_view, node);

        build_plans.insert(node, build_plan);
        provides.insert(node, out_schema);
        compiled_plans.insert(node, plan);

        map_tables_to_node(node, &mut in_view);
    }
    // debug_build_plans(&build_plans, &provides);
    let mut views = HashMap::default();
    let mut tables_buf = Vec::default();
    let mut total_building = Duration::default();
    let mut total_joining = Duration::default();
    let start = Instant::now();
    // TODO hash treeop by address
    for (node, compiled_plan) in &compiled_plans {
        let loop_start = Instant::now();
        let build_plan = &build_plans[node];

        let build_start = Instant::now();
        let tables = build_tables(db, &views, build_plan, build_strategy);
        let build_time = build_start.elapsed();
        println!("Building takes {}", build_time.as_secs_f32());
        total_building += build_time;

        let mut intermediate = View {
            vec: vec![],
            arity: provides[node].len(),
        };

        println!("Running join with {} tables", tables.len());
        let join_start = Instant::now();
        free_join(
            vectorize,
            optimize,
            &tables,
            compiled_plan,
            &mut intermediate,
        );
        let join_time = join_start.elapsed();
        println!("Join took {:?}", join_time.as_secs_f32());
        total_joining += join_time;

        views.insert(node, intermediate);
        tables_buf.push(tables);
        println!("Iter takes {:?}", loop_start.elapsed().as_secs_f32());
    }
    println!("Bushy join takes {:?}", start.elapsed().as_secs_f32());
    let final_attrs = provides.get(&root).unwrap();
    let final_view = views.get(&root).unwrap();
    println!("out rows: {}", final_view.vec.len() / final_view.arity);
    print!("output ");
    let payload_ids: Vec<_> = payload
        .iter()
        .map(|p| {
            final_attrs
                .iter()
                .position(|attrs| attrs.contains(p))
                .unwrap()
        })
        .collect();
    let mut result = Vec::default();
    // for row in final_view {
    //     if result.is_empty() {
    //         result = row.to_vec();
    //     } else {
    //         for (i, v) in row.iter().enumerate() {
    //             if &result[i] > v {
    //                 result[i] = v.clone();
    //             }
    //         }
    //     }
    // }
    for row in final_view {
        if result.is_empty() {
            result = payload_ids.iter().map(|i| &row[*i]).collect();
        } else {
            for (j, i) in payload_ids.iter().enumerate() {
                if result[j] > &row[*i] {
                    result[j] = &row[*i];
                }
            }
        }
    }
    println!("{:?}", result);
    println!("Total building takes {}", total_building.as_secs_f32());
    println!("Total joining takes {}", total_joining.as_secs_f32());
    let total = start.elapsed();
    println!("Total takes {}", total.as_secs_f32());

    total.as_secs_f64()
    // use trie::*;
    // let mut stats = HashMap::default();
    // for tables in tables_buf {
    //     for table in tables {
    //         if let Tb::Trie(t) = table {
    //             t.compute_trie_stats(&mut stats)
    //         }
    //     }
    // }
    // let mut stats = stats
    //     .into_iter()
    //     .map(|(len, amt)| (amt, len))
    //     .collect::<Vec<_>>();
    // stats.sort_unstable();
    // stats.reverse();
    // stats.truncate(50);
    // log::debug!("trie stats: {:?}", stats);
}

// mapping between the original query ID to duckdb's ID
fn queries() -> IndexMap<&'static str, &'static str> {
    // let queries = vec![("11a", "IMDBQ113")];

    // return queries.into_iter().collect();

    let bushy = true;
    let linear = true;

    let mut queries = vec![];

    if linear {
        queries.extend_from_slice(&[
            ("1a", "IMDBQ001"),
            ("1b", "IMDBQ002"),
            ("1c", "IMDBQ003"),
            ("1d", "IMDBQ004"),
            ("2a", "IMDBQ005"),
            ("2b", "IMDBQ006"),
            // ("2c", "IMDBQ007"), // EMPTY
            ("2d", "IMDBQ008"),
            ("3a", "IMDBQ009"),
            ("3b", "IMDBQ010"),
            ("3c", "IMDBQ011"),
            ("4a", "IMDBQ012"),
            ("4b", "IMDBQ013"),
            ("4c", "IMDBQ014"),
            // EMPTY input ("5a", "IMDBQ015"),
            // EMPTY input ("5b", "IMDBQ016"),
            ("5c", "IMDBQ017"),
        ])
    }

    if bushy {
        queries.extend_from_slice(&[
            ("6a", "IMDBQ018"),
            ("6b", "IMDBQ019"),
            ("6c", "IMDBQ020"),
            ("6d", "IMDBQ021"),
            ("6e", "IMDBQ022"),
            ("6f", "IMDBQ023"),
            ("7a", "IMDBQ024"),
            ("7b", "IMDBQ025"),
            ("7c", "IMDBQ026"),
            ("8a", "IMDBQ027"),
            ("8b", "IMDBQ028"),
            ("8c", "IMDBQ029"), // SLOW
            ("8d", "IMDBQ030"), // SLOW
            ("9a", "IMDBQ031"),
            ("9b", "IMDBQ032"),
            ("9c", "IMDBQ033"),
            ("9d", "IMDBQ034"), // SLOW
            ("10a", "IMDBQ035"),
            // ("10b", "IMDBQ036"), // EMPTY
            ("10c", "IMDBQ037"),
        ])
    }

    if linear {
        queries.extend_from_slice(&[
            ("11a", "IMDBQ038"),
            ("11b", "IMDBQ039"),
            ("11c", "IMDBQ040"),
            ("11d", "IMDBQ041"),
            ("12a", "IMDBQ042"),
            ("12b", "IMDBQ043"), // TRIE SLOW
            ("12c", "IMDBQ044"),
            // ("13a", "IMDBQ045"),
            ("13b", "IMDBQ046"), // TRIE SLOW
            ("13c", "IMDBQ047"), // TRIE SLOW
            // ("13d", "IMDBQ048"),
            ("14a", "IMDBQ049"),
            ("14b", "IMDBQ050"),
            ("14c", "IMDBQ051"),
            ("15a", "IMDBQ052"),
            ("15b", "IMDBQ053"),
            ("15c", "IMDBQ054"),
            ("15d", "IMDBQ055"),
        ])
    }

    if bushy {
        queries.extend_from_slice(&[
            ("16a", "IMDBQ056"),
            ("16b", "IMDBQ057"),
            ("16c", "IMDBQ058"),
            ("16d", "IMDBQ059"),
            ("17a", "IMDBQ060"),
            ("17b", "IMDBQ061"),
            ("17c", "IMDBQ062"),
            ("17d", "IMDBQ063"),
            ("17e", "IMDBQ064"),
            ("17f", "IMDBQ065"),
            ("18a", "IMDBQ066"),
            ("18b", "IMDBQ067"),
            ("18c", "IMDBQ068"),
            ("19a", "IMDBQ069"),
            ("19b", "IMDBQ070"),
            ("19c", "IMDBQ071"),
            ("19d", "IMDBQ072"),
            ("20a", "IMDBQ073"),
            ("20b", "IMDBQ074"),
            ("20c", "IMDBQ075"),
        ])
    }

    if linear {
        queries.extend_from_slice(&[
            ("21a", "IMDBQ076"),
            ("21b", "IMDBQ077"),
            ("21c", "IMDBQ078"),
        ])
    }

    if bushy {
        queries.extend_from_slice(&[
            ("22a", "IMDBQ079"),
            ("22b", "IMDBQ080"),
            ("22c", "IMDBQ081"),
            ("22d", "IMDBQ082"),
            ("23a", "IMDBQ083"), // SLOW
            ("23b", "IMDBQ084"),
            ("23c", "IMDBQ085"), // SLOW
            ("24a", "IMDBQ086"),
            ("24b", "IMDBQ087"),
            ("25a", "IMDBQ088"),
            ("25b", "IMDBQ089"),
            ("25c", "IMDBQ090"),
            ("26a", "IMDBQ091"),
            ("26b", "IMDBQ092"),
            ("26c", "IMDBQ093"),
            ("27a", "IMDBQ094"),
            ("27b", "IMDBQ095"),
            ("27c", "IMDBQ096"),
            ("28a", "IMDBQ097"),
            ("28b", "IMDBQ098"),
            ("28c", "IMDBQ099"),
            ("29a", "IMDBQ100"),
            ("29b", "IMDBQ101"),
            ("29c", "IMDBQ102"),
            ("30a", "IMDBQ103"),
            ("30b", "IMDBQ104"),
            ("30c", "IMDBQ105"),
            ("31a", "IMDBQ106"),
            ("31b", "IMDBQ107"),
            ("31c", "IMDBQ108"),
        ])
    }

    if linear {
        queries.extend_from_slice(&[
            // ("32a", "IMDBQ109"), // TRIE SLOW // EMPTY
            ("32b", "IMDBQ110"),
        ])
    }

    if bushy {
        queries.extend_from_slice(&[
            ("33a", "IMDBQ111"), // SLOW
            ("33b", "IMDBQ112"), // SLOW
            ("33c", "IMDBQ113"), // SLOW
        ])
    }

    // */
    queries.into_iter().collect()
}
