use super::*;
use bytebuffer::*;
use Position::*;

fn print_tree<T: std::fmt::Display>(tree: &Tree<T>){
    if let Some(root) = tree.get_root(){
        for node in tree.sub_tree_from_counted(root).unwrap(){
            println!("\"{}\" child count: {}", tree.data_at(node.id).unwrap(), node.child_count);
        }
    }
    else{
        println!("Empty");
    }
    println!("");
}

fn tree_matches<T: std::fmt::Display + PartialEq>(tree: &Tree<T>, expected: Vec<(T, usize)>) -> bool{
    print_tree(&tree);

    assert_eq!(tree.len(), expected.len());

    if let Some(root) = tree.get_root(){
        for (node_children, expected) in tree.sub_tree_from_counted(root).unwrap().iter().zip(expected.iter()){
            if tree.data_at(node_children.id).unwrap() != &expected.0 || node_children.child_count != expected.1 { return false }
        }
    }

    true
}

const ROOT_STR: &str = "root";
const ROOT_ID: usize = 0;
const FIRST_ROOT_CHILD_STR: &str = "1st root child";
const FIRST_ROOT_CHILD_ID: usize = 1;
const LAST_ROOT_CHILD_STR: &str = "2nd root child";
const LAST_ROOT_CHILD_ID: usize = 2;

fn make_tree() -> Tree<&'static str>{
    let mut tree = Tree::new_with_root(ROOT_STR);

    tree.new_node(FIRST_ROOT_CHILD_STR, LastChild, ROOT_ID).unwrap();

    tree.new_node(LAST_ROOT_CHILD_STR, LastChild, ROOT_ID).unwrap();

    tree
}

#[test]
fn base_tree() {
    let tree = make_tree();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));
}

#[test]
fn append_child(){
    let mut tree = make_tree();

    let new1 = "append1";
    let new2 = "append2";

    let new1_id = tree.new_node(new1, LastChild, FIRST_ROOT_CHILD_ID).unwrap();
    let new2_id = tree.new_node(new2, LastChild, FIRST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 2),
        (new1, 0),
        (new2, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    // parent links
    assert_eq!(tree.first_child_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new1_id));
    assert_eq!(tree.last_child_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new2_id));

    // child links
    assert_eq!(tree.parent_of(new1_id).unwrap(), Some(FIRST_ROOT_CHILD_ID));
    assert_eq!(tree.prev_sib_of(new1_id).unwrap(), None);
    assert_eq!(tree.next_sib_of(new1_id).unwrap(), Some(new2_id));
    assert_eq!(tree.first_child_of(new1_id).unwrap(), None);
    assert_eq!(tree.last_child_of(new1_id).unwrap(), None);

    assert_eq!(tree.prev_sib_of(new2_id).unwrap(), Some(new1_id));
    assert_eq!(tree.next_sib_of(new2_id).unwrap(), None);
}

#[test]
fn prepend_child(){
    let mut tree = make_tree();

    let new1 = "prepend1";
    let new2 = "prepend2";

    let new1_id = tree.new_node(new1, FirstChild, FIRST_ROOT_CHILD_ID).unwrap();
    let new2_id = tree.new_node(new2, FirstChild, FIRST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 2),
        (new2, 0),
        (new1, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    // parent links
    assert_eq!(tree.first_child_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new2_id));
    assert_eq!(tree.last_child_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new1_id));

    // child links
    assert_eq!(tree.parent_of(new1_id).unwrap(), Some(FIRST_ROOT_CHILD_ID));
    assert_eq!(tree.first_child_of(new1_id).unwrap(), None);
    assert_eq!(tree.last_child_of(new1_id).unwrap(), None);

    assert_eq!(tree.prev_sib_of(new1_id).unwrap(), Some(new2_id));
    assert_eq!(tree.next_sib_of(new1_id).unwrap(), None);

    assert_eq!(tree.prev_sib_of(new2_id).unwrap(), None);
    assert_eq!(tree.next_sib_of(new2_id).unwrap(), Some(new1_id));
}

#[test]
fn sibling_before() {
    let mut tree = make_tree();

    let new1 = "sib before1";
    let new2 = "sib before2";

    let new1_id = tree.new_node(new1, SiblingBefore, FIRST_ROOT_CHILD_ID).unwrap();
    let new2_id = tree.new_node(new2, SiblingBefore, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 4),
        (new1, 0),
        (FIRST_ROOT_CHILD_STR, 0),
        (new2, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    // parent links
    assert_eq!(tree.first_child_of(ROOT_ID).unwrap(), Some(new1_id));
    assert_eq!(tree.prev_sib_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new1_id));
    assert_eq!(tree.next_sib_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new2_id));
    assert_eq!(tree.prev_sib_of(LAST_ROOT_CHILD_ID).unwrap(), Some(new2_id));

    // child links
    assert_eq!(tree.parent_of(new1_id).unwrap(), Some(ROOT_ID));
    assert_eq!(tree.first_child_of(new1_id).unwrap(), None);
    assert_eq!(tree.last_child_of(new1_id).unwrap(), None);

    assert_eq!(tree.prev_sib_of(new1_id).unwrap(), None);
    assert_eq!(tree.next_sib_of(new1_id).unwrap(), Some(FIRST_ROOT_CHILD_ID));

    assert_eq!(tree.prev_sib_of(new2_id).unwrap(), Some(FIRST_ROOT_CHILD_ID));
    assert_eq!(tree.next_sib_of(new2_id).unwrap(), Some(LAST_ROOT_CHILD_ID));
}

#[test]
fn sibling_after() {
    let mut tree = make_tree();

    let new1 = "sib after1";
    let new2 = "sib after2";

    let new1_id = tree.new_node(new1, SiblingAfter, FIRST_ROOT_CHILD_ID).unwrap();
    let new2_id = tree.new_node(new2, SiblingAfter, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 4),
        (FIRST_ROOT_CHILD_STR, 0),
        (new1, 0),
        (LAST_ROOT_CHILD_STR, 0),
        (new2, 0),
    ]));

    // parent links
    assert_eq!(tree.last_child_of(ROOT_ID).unwrap(), Some(new2_id));
    assert_eq!(tree.next_sib_of(FIRST_ROOT_CHILD_ID).unwrap(), Some(new1_id));
    assert_eq!(tree.prev_sib_of(LAST_ROOT_CHILD_ID).unwrap(), Some(new1_id));
    assert_eq!(tree.next_sib_of(LAST_ROOT_CHILD_ID).unwrap(), Some(new2_id));

    // child links
    assert_eq!(tree.parent_of(new1_id).unwrap(), Some(ROOT_ID));
    assert_eq!(tree.first_child_of(new1_id).unwrap(), None);
    assert_eq!(tree.last_child_of(new1_id).unwrap(), None);

    assert_eq!(tree.prev_sib_of(new1_id).unwrap(), Some(FIRST_ROOT_CHILD_ID));
    assert_eq!(tree.next_sib_of(new1_id).unwrap(), Some(LAST_ROOT_CHILD_ID));

    assert_eq!(tree.prev_sib_of(new2_id).unwrap(), Some(LAST_ROOT_CHILD_ID));
    assert_eq!(tree.next_sib_of(new2_id).unwrap(), None);
}

#[test]
fn copy_node() {
    let mut tree = make_tree();

    tree.clone_to(ROOT_ID, FirstChild, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 1),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));
}

#[test]
fn move_node() {
    let mut tree = make_tree();

    let clone_id = tree.clone_to(ROOT_ID, FirstChild, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 1),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    tree.move_to(clone_id, FirstChild, ROOT_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 3),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));
}

#[test]
fn remove_node() {
    let mut tree = make_tree();

    tree.clone_to(ROOT_ID, FirstChild, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 1),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    tree.remove(LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 1),
        (FIRST_ROOT_CHILD_STR, 0),
    ]));
}

#[test]
fn new_root() {
    let mut tree = make_tree();

    let new = "new root";

    tree.new_root(new);

    assert!(tree_matches(&tree, vec![
        (new, 0),
    ]));
}

#[test]
fn change_root() {
    let mut tree = make_tree();

    tree.clone_to(ROOT_ID, FirstChild, LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 1),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));

    tree.make_root(LAST_ROOT_CHILD_ID).unwrap();

    assert!(tree_matches(&tree, vec![
        (LAST_ROOT_CHILD_STR, 1),
        (ROOT_STR, 2),
        (FIRST_ROOT_CHILD_STR, 0),
        (LAST_ROOT_CHILD_STR, 0),
    ]));
}

#[test]
fn bytes(){
    let mut tree = Tree::new();

    println!("empty tree:");
    assert!(tree_matches(&tree, vec![]));

    let new_tree = Tree::<i32>::from_bytes(&mut tree.into_bytes()).unwrap();

    println!("new empty tree:");
    assert!(tree_matches(&new_tree, vec![]));

    tree.new_root(0);
    tree.new_node(10, LastChild, 0).unwrap();

    let node2 = tree.new_node(20, LastChild, 0).unwrap();
    tree.new_node(21, LastChild, node2).unwrap();
    tree.new_node(22, LastChild, node2).unwrap();
    tree.new_node(23, LastChild, node2).unwrap();
    
    tree.new_node(30, LastChild, 0).unwrap();

    println!("filled tree:");
    assert!(tree_matches(&tree, vec![
        (0, 3),
        (10, 0),
        (20, 3),
        (21, 0),
        (22, 0),
        (23, 0),
        (30, 0),
    ]));

    let new_tree = Tree::<i32>::from_bytes(&mut tree.into_bytes()).unwrap();

    println!("new filled tree:");
    assert!(tree_matches(&new_tree, vec![
        (0, 3),
        (10, 0),
        (20, 3),
        (21, 0),
        (22, 0),
        (23, 0),
        (30, 0),
    ]));
}