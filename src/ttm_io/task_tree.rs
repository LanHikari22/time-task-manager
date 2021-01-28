//! developed by Mohammed Alzakariya (lanhikarixx@gmail.com)
//! This module iterates through tabbed Task items and parses out
//! a tree model to represent them.
//! TODO implement module

// TODO until implemented
#![allow(dead_code)]
#![allow(unused_imports)]

use super::task;
use crate::utils::common;
use trees::{fr, tr};

#[derive(Debug)]
enum TaskTreeNode {
    Task(task::Task),
    Label(String),
    Commit(String),
}

#[derive(Debug)]
enum TaskTreeParseError {
    TaskParseError(task::TaskParseError),
    Todo,
}

struct TaskTree {
    pub tree: trees::Tree<TaskTreeNode>,
}

impl std::str::FromStr for TaskTree {
    type Err = TaskTreeParseError;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut out = TaskTree {
            tree: tr(TaskTreeNode::Label("root".into())),
        };

        // state to populate the tree
        let mut last_tab_level = 0;
        let mut root = out.tree;
        let mut cur_node = root.root_mut();
        let mut walk = trees::TreeWalk::from(root);

        for line in s.lines() {
            let tab_level = common::StrUtils(line).tabs().len();
            
            // parse the node
            let node: Option<TaskTreeNode> = line
                .parse::<task::Task>()
                .ok()
                .map(|_task| TaskTreeNode::Task(_task));

            // insert the node into the tree
            if let Some(_node) = node {
                if tab_level > last_tab_level {
                    // node is a child of last node
                    cur_node = cur_node.push_back(tr(_node));
                    walk.get().unwrap().node().push_front(tr(_node));
                    walk.to_child(0);
                } else if tab_level < last_tab_level {
                    // node is a sibling of parent
                    match _grandparent(walk.get().unwrap().node()) {
                        Some(grandparent_node) => {
                            grandparent_node.push_front(tr(_node));
                            walk.to_parent();
                            walk.to_sib(0);
                        }
                        None => return Err(TaskTreeParseError::Todo),
                    }
                } else {
                    // node is a sibling
                    match root.parent() {
                        Some(parent_node) => {
                            root = tr(_node);
                            parent_node.push_back(root);
                        }
                        None => return Err(TaskTreeParseError::Todo),
                    }
                }
            }

            last_tab_level = tab_level;

            println!("{:?}", out.tree);
        }
        todo!()
    }
}

fn _grandparent(node: &trees::Node<TaskTreeNode>) -> Option<&trees::Node<TaskTreeNode>> {
    Some(node.parent()?.parent()?)
}

fn _insertNodeAsChildOrSibling(tree: &mut TaskTree, node: &TaskTreeNode) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;

    #[test]
    fn test_parse_only_tasks() {
        let td_tasktree_0 = r#"
() :ttm
    (2,0,0) Task 10 (*P[Y21W-W3U-2321])
        (1,0) Task 11
        S(0,5,5) Task 12
    B(0,5,10) Task 20
        ~(1) Task 21
    (0,0) Task 01 "#;

        let res = td_tasktree_0.parse::<TaskTree>();
        println!("{:?}", res.unwrap().tree);
        panic!("entry point");
    }

    #[test]
    fn test_parse_general() {
        println!("hiii");
    }

    #[test]
    fn test_parse_entrypoint() {
        let td_tasktree_0 = r#"
() :ttm
  Label1
    (2,0,0) Task 10 (*P[Y21W-W3U-2321])
    (1,0) Task 11
    S(0,5,5) Task 12
  Label2
    B(0,5,10) Task 20
      N- Blocked: Reason
    ~(1) Task 21
      - Commit 1
  (0,0) Task 01 "#;

        let res = td_tasktree_0.parse::<TaskTree>();
        println!("{:?}", res.unwrap().tree);
        panic!("entry point");
    }
}
