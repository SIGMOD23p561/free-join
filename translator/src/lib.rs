#![allow(warnings)]
use crate::JoinType::Inner;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    Mark,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub attr_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Equalizer {
    pub left_attr: Attribute,
    pub right_attr: Attribute,
}

// Force to rename "extra-info" into "extra_info"
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinAttr {
    pub join_type: JoinType,
    pub equalizers: Vec<Equalizer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanAttr {
    pub table_name: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeAttr {
    Join(JoinAttr),
    Scan(ScanAttr),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TreeOp {
    pub name: String,
    pub cardinality: i32,
    pub extra_info: String,
    pub children: Vec<Box<TreeOp>>,
    pub attr: Option<NodeAttr>,
}

pub struct TraverseFuncs<'a, MR, RR, CR> {
    pub map_func: &'a dyn Fn(&TreeOp) -> MR,
    pub combine_func: &'a dyn Fn(MR, RR) -> CR,
    pub reduce_func: &'a dyn Fn(RR, CR) -> RR,
    pub default_func: &'a dyn Fn() -> RR,
}

pub fn traverse<R, S, T>(node: &TreeOp, traverse_funcs: &TraverseFuncs<R, S, T>) -> T {
    let map_result: R = (traverse_funcs.map_func)(node);
    let reduce_result: S = node
        .children
        .iter()
        .map(|child_node| traverse(child_node, traverse_funcs))
        .fold((traverse_funcs.default_func)(), |a: S, b: T| -> S {
            (traverse_funcs.reduce_func)(a, b)
        });
    (traverse_funcs.combine_func)(map_result, reduce_result)
}

pub fn preorder_traverse_mut<T>(node: &mut TreeOp, func: &mut T)
where
    T: FnMut(&mut TreeOp),
{
    func(node);
    for child_node in node.children.iter_mut() {
        preorder_traverse_mut(child_node, func);
    }
}

pub fn postorder_traverse_mut<T>(node: &mut TreeOp, func: &mut T)
where
    T: FnMut(&mut TreeOp),
{
    for child_node in node.children.iter_mut() {
        postorder_traverse_mut(child_node, func);
    }
    func(node);
}

pub fn parse_tree_extra_info(root: &mut TreeOp, filename: &str) {
    let mut parse_func = |node: &mut TreeOp| match node.name.as_str() {
        "HASH_JOIN" => {
            let extra_info: Vec<_> = node
                .extra_info
                .split('\n')
                .filter(|s| !s.is_empty())
                .collect();

            let join_type = match &extra_info[0] {
                &"INNER" => JoinType::Inner,
                &"MARK" => {
                    eprintln!("{} contains MARK Join", filename);
                    JoinType::Mark
                }
                _ => panic!("Fail to parse Join Type {}", extra_info[0]),
            };

            let mut equalizers = Vec::new();

            for pred in &extra_info[1..] {
                let equalizer = pred.split('=').map(|s| s.trim()).collect::<Vec<_>>();
                equalizers.push(Equalizer {
                    left_attr: Attribute {
                        attr_name: equalizer[0].to_string(),
                    },
                    right_attr: Attribute {
                        attr_name: equalizer[1].to_string(),
                    },
                });
            }

            node.attr = Some(NodeAttr::Join(JoinAttr {
                join_type,
                equalizers,
            }));
        }
        "SEQ_SCAN" => {
            let extra_info = node.extra_info.replace("[INFOSEPARATOR]", "");
            let info_strs: Vec<_> = extra_info.split('\n').filter(|s| !s.is_empty()).collect();

            let table_name: String = info_strs.first().expect("Failed to Get Table").to_string();
            node.attr = Some(NodeAttr::Scan(ScanAttr {
                table_name,
                attributes: vec![],
            }));
        }
        _ => (),
    };
    preorder_traverse_mut(root, &mut parse_func);
}

pub fn to_gj_plan(root: &mut TreeOp) -> Vec<Vec<String>> {
    let mut plan: Vec<Vec<String>> = vec![];

    let mut get_plan = |node: &mut TreeOp| {
        if let Some(NodeAttr::Join(attr)) = &node.attr {
            if attr.join_type == Inner {
                for equalizer in &attr.equalizers {
                    let lattr_name = equalizer.left_attr.attr_name.clone();
                    let rattr_name = equalizer.right_attr.attr_name.clone();

                    // Find in plan the index of vector which contains attr_name;
                    let lpos_opt = plan.iter().position(|x| x.contains(&lattr_name));
                    let rpos_opt = plan.iter().position(|x| x.contains(&rattr_name));

                    // We have four cases and enumerate
                    match (lpos_opt, rpos_opt) {
                        (Some(lpos), Some(rpos)) => assert_eq!(lpos, rpos),
                        (Some(lpos), None) => plan[lpos].push(rattr_name),
                        (None, Some(rpos)) => plan[rpos].push(lattr_name),
                        (None, None) => {
                            plan.push(vec![]);
                            plan.last_mut().unwrap().push(lattr_name);
                            plan.last_mut().unwrap().push(rattr_name);
                        }
                    }
                }
            }
        }
    };

    postorder_traverse_mut(root, &mut get_plan);

    plan
}
