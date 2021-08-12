//! Tree is a generic collection type that allows to crawling around using the relations of nodes or 
//! jumping to specific nodes with ids.
//! 
//! Implements all bytebuffer traits.
//! 
//! ## Structs
//! - Tree
//! - NodeChildren
//! - TreeIter
//! 
//! ## Enums
//! - Position
//! - TreeErr

use bytebuffer::*;

/// The individual nodes on the tree.
struct Node<T> {
    parent: Option<usize>,

    prev_sib: Option<usize>,
    next_sib: Option<usize>,

    first_child: Option<usize>,
    last_child: Option<usize>,

    data: Option<T>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node{
            parent:      None,
            prev_sib:    None,
            next_sib:    None,
            first_child: None,
            last_child:  None,
            data:        Some(data),
        }
    }
}

/// The id of a node along with the number of children that it has.
#[derive(Clone, Copy)]
pub struct NodeInfo{
    pub id: usize,
    pub child_count: usize,
    pub depth: usize,
}

/// The positions that a node can be placed in relation to another node.
pub enum Position {
    FirstChild,
    LastChild,
    SiblingBefore,
    SiblingAfter,
}

use Position::*;

/// The errors that can be returned from Tree functions.
#[derive(Debug)]
pub enum TreeErr{
    InvalidId,
    CantBeRoot,
    CantMoveIntoChild,
}

use TreeErr::*;

/// A collection of nodes and there relations.
/// 
/// ## Functions
/// - new
/// - new_with_root
/// 
/// ## Methods
/// - len
/// - descendants_of
/// - sub_tree
/// - sub_tree_info
/// - sub_tree_depth
/// - sub_tree_depth_info
/// - sub_tree_depth
/// - sub_tree_depth_info
/// - children_of
/// - new_node
/// - remove
/// - data_at
/// - data_at_mut
/// - get_root
/// - new_root
/// - make_root
/// - parent_of
/// - next_sib_of
/// - prev_sib_of
/// - first_child_of
/// - last_child_of
/// - move_to
/// ### if impl Copy + Clone
/// - clone_to
/// ### if impl IntoBytes
/// - into_bytes
/// ### if impl FromBytes
/// - from_bytes
/// - from_io_bytes
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    free: Option<usize>,
    root: Option<usize>,
    len: usize,
}

impl<T> Tree<T> {
    /// Creates an empty tree.
    pub fn new() -> Tree<T> {
        Tree {
            free: None,
            nodes: Vec::new(),
            root: None,
            len: 0,
        }
    }

    /// Creates a tree with the provided data as the tree root with an id of zero.
    pub fn new_with_root(data: T) -> Tree<T> {
        Tree {
            free: None,
            nodes: vec![
                Node::new(data),
            ],
            root: Some(0),
            len: 1,
        }
    }

    fn valid_node(&self, id: usize) -> Result<(), TreeErr> {
        if id >= self.nodes.len() { return Err(InvalidId) }
        if self.nodes[id].data.is_none() { return Err(InvalidId) }
        Ok(())
    }

    fn valid_sib(&self, id: usize) -> Result<(), TreeErr> {
        if id >= self.nodes.len() { return Err(InvalidId) }
        if self.nodes[id].data.is_none() { return Err(InvalidId) }
        if id == self.root.unwrap() { return Err(CantBeRoot) }
        Ok(())
    }

    fn push_free(&mut self, id: usize) {
        self.nodes[id].parent      = None;
        self.nodes[id].prev_sib    = None;
        self.nodes[id].next_sib    = self.free;
        self.nodes[id].first_child = None;
        self.nodes[id].last_child  = None;
        self.nodes[id].data        = None;

        self.free = Some(id);
        self.len -= 1;
    }

    fn pop_free(&mut self) -> Option<usize> {
        if let Some(id) = self.free {
            self.free = self.nodes[id].next_sib;
            self.nodes[id].next_sib = None;

            return Some(id);
        }

        None
    }

    fn get_node(&mut self, data: T) -> usize {
        self.len += 1;
        
        if let Some(id) = self.pop_free() {
            self.nodes[id].data = Some(data);

            return id;
        }
        else {
            self.nodes.push(Node::new(data));

            return self.nodes.len() - 1;
        }
    }

    fn append_child(&mut self, parent_id: usize, new_id: usize) {
        //previous sibling of new set to the parents last child.
        self.nodes[new_id].prev_sib = self.nodes[parent_id].last_child;
        if let Some(prev) = self.nodes[new_id].prev_sib {
            self.nodes[prev].next_sib = Some(new_id);
        }

        self.nodes[new_id].parent = Some(parent_id);

        //last child of parent updated to be the new node.
        self.nodes[parent_id].last_child = Some(new_id);

        //if the parent didn't have any children new is set also set to the first child.
        if self.nodes[parent_id].first_child.is_none() {
            self.nodes[parent_id].first_child = Some(new_id);
        }
    }

    fn prepend_child(&mut self, parent_id: usize, new_id: usize){
        //next sibling of new set to the parents first child.
        self.nodes[new_id].next_sib = self.nodes[parent_id].first_child;
        if let Some(next) = self.nodes[new_id].next_sib {
            self.nodes[next].prev_sib = Some(new_id);
        }

        self.nodes[new_id].parent = Some(parent_id);

        //first child of parent updated to be the new node.
        self.nodes[parent_id].first_child = Some(new_id);

        //if the parent didn't have any children new is set also set to the last child.
        if self.nodes[parent_id].last_child.is_none() {
            self.nodes[parent_id].last_child = Some(new_id);
        }
    }

    fn add_sibling_before(&mut self, sibling_id: usize, new_id: usize) {
        self.nodes[new_id].next_sib = Some(sibling_id);
        self.nodes[new_id].prev_sib = self.nodes[sibling_id].prev_sib;
        self.nodes[new_id].parent = self.nodes[sibling_id].parent;

        self.nodes[sibling_id].prev_sib = Some(new_id);
        
        if let Some(prev_sib_id) = self.nodes[new_id].prev_sib {
            self.nodes[prev_sib_id].next_sib = Some(new_id);
        }
        else if let Some(parent_id) = self.nodes[new_id].parent {
            self.nodes[parent_id].first_child = Some(new_id);
        }
    }

    fn add_sibling_after(&mut self, sibling_id: usize, new_id: usize) {
        self.nodes[new_id].prev_sib = Some(sibling_id);
        self.nodes[new_id].next_sib = self.nodes[sibling_id].next_sib;
        self.nodes[new_id].parent = self.nodes[sibling_id].parent;

        self.nodes[sibling_id].next_sib = Some(new_id);

        if let Some(next_sib_id) = self.nodes[new_id].next_sib {
            self.nodes[next_sib_id].prev_sib = Some(new_id);
        }
        else if let Some(parent_id) = self.nodes[new_id].parent {
            self.nodes[parent_id].last_child = Some(new_id);
        }
    }

    fn attach(&mut self, attaching: usize, in_position: Position, node: usize){
        match in_position {
            FirstChild    => self.prepend_child(node, attaching),
            LastChild     => self.append_child(node, attaching),
            SiblingBefore => self.add_sibling_before(node, attaching),
            SiblingAfter  => self.add_sibling_after(node, attaching)
        }
    }

    fn decouple(&mut self, id: usize){
        if let Some(prev) = self.nodes[id].prev_sib {
            self.nodes[prev].next_sib = self.nodes[id].next_sib;
        }
        else if let Some(parent) = self.nodes[id].parent {
            self.nodes[parent].first_child = self.nodes[id].next_sib;
        }

        if let Some(next) = self.nodes[id].next_sib {
            self.nodes[next].prev_sib = self.nodes[id].prev_sib;
        }
        else if let Some(parent) = self.nodes[id].parent {
            self.nodes[parent].last_child = self.nodes[id].prev_sib;
        }
    }

    /// Returns the number of nodes currently in the tree.
    pub fn len(&self) -> usize {
        self.len
    }

    fn descendants_of_helper(&self, id: usize, ids: &mut Vec<usize>){
        let mut child = self.nodes[id].first_child;

        while let Some(child_id) = child {
            ids.push(child_id);
            self.descendants_of_helper(child_id, ids);
            child = self.nodes[child_id].next_sib;
        }
    }

    /// Returns a list of all of the descendants of the provided id.
    pub fn descendants_of(&self, id: usize) -> Result<Vec<usize>, TreeErr> {
        self.valid_node(id)?;

        let mut ids = Vec::with_capacity(self.len());
        self.descendants_of_helper(id, &mut ids);

        Ok(ids)
    }

    /// Returns a list starting with the id provided followed by all of its descendants.
    pub fn sub_tree(&self, id: usize) -> Result<Vec<usize>, TreeErr> {
        self.valid_node(id)?;

        let mut ids = Vec::with_capacity(self.len());
        ids.push(id);
        self.descendants_of_helper(id, &mut ids);

        Ok(ids)
    }

    fn sub_tree_info_helper(&self, id: usize, ids: &mut Vec<NodeInfo>, cur_depth: usize){
        let index = ids.len();
        ids.push(NodeInfo{
            id: id,
            child_count: 0,
            depth: cur_depth,
        });

        let mut child = self.nodes[id].first_child;

        while let Some(child_id) = child {
            ids[index].child_count += 1;
            self.sub_tree_info_helper(child_id, ids, cur_depth + 1);
            child = self.nodes[child_id].next_sib;
        }
    }

    /// Returns a list starting with the id provided and the number of children it has followed by the same for all of its descendants.
    pub fn sub_tree_info(&self, id: usize) -> Result<Vec<NodeInfo>, TreeErr> {
        self.valid_node(id)?;

        let mut ids = Vec::new();
        self.sub_tree_info_helper(id, &mut ids, 0);

        Ok(ids)
    }

    fn sub_tree_depth_helper(&self, id: usize, ids: &mut Vec<usize>, depth: usize){
        let mut child = self.nodes[id].first_child;

        while let Some(child_id) = child {
            ids.push(child_id);

            if depth > 0 {
                self.sub_tree_depth_helper(child_id, ids, depth - 1);
            }

            child = self.nodes[child_id].next_sib;
        }
    }

    /// Returns a list starting with the id provided followed by all of its descendants up to the given depth.
    pub fn sub_tree_depth(&self, id: usize, depth: usize) -> Result<Vec<usize>, TreeErr> {
        self.valid_node(id)?;

        let mut ids = Vec::with_capacity(self.len());
        ids.push(id);

        if depth > 0 {
            self.sub_tree_depth_helper(id, &mut ids, depth - 1);
        }
        
        Ok(ids)
    }

    fn sub_tree_depth_info_helper(&self, id: usize, ids: &mut Vec<NodeInfo>, cur_depth: usize, target: usize){
        let index = ids.len();
        ids.push(NodeInfo{
            id: id,
            child_count: 0,
            depth: cur_depth,
        });

        let mut child = self.nodes[id].first_child;

        while let Some(child_id) = child {
            ids[index].child_count += 1;

            if cur_depth < target {
                self.sub_tree_depth_info_helper(child_id, ids, cur_depth + 1, target);
            }

            child = self.nodes[child_id].next_sib;
        }
    }

    /// Returns a list starting with the id provided and the number of children it has followed by the same for all of its descendants up to the given depth.
    pub fn sub_tree_depth_info(&self, id: usize, depth: usize) -> Result<Vec<NodeInfo>, TreeErr> {
        self.valid_node(id)?;

        let mut ids = Vec::new();

        self.sub_tree_depth_info_helper(id, &mut ids, 0, depth);

        Ok(ids)
    }

    /// Returns a list of all of the child ids of the given node.
    pub fn children_of(&self, id: usize) -> Result<Vec<usize>, TreeErr>{
        self.valid_node(id)?;

        let mut children = Vec::new();
        let mut child = self.first_child_of(id).unwrap();

        while let Some(child_id) = child{
            children.push(child_id);
            child = self.next_sib_of(child_id).unwrap();
        }

        Ok(children)
    }

    /// Creates a new node containing the data provide and attaches it to the node provide in the position of in_position.
    pub fn new_node(&mut self, data: T, in_position: Position, node: usize) -> Result<usize, TreeErr> {
        match in_position {
            FirstChild    | LastChild     => self.valid_node(node)?,
            SiblingBefore | SiblingAfter  => self.valid_sib(node)?
        }

        let new = self.get_node(data);
        self.attach(new, in_position, node);
        
        Ok(new)
    }

    /// Removes a node from the tree along with all of its descendants.
    pub fn remove(&mut self, id: usize) -> Result<(), TreeErr> {
        self.valid_node(id)?;

        self.decouple(id);

        for child in self.descendants_of(id).unwrap() {
            self.push_free(child);
        }

        self.push_free(id);

        Ok(())
    }

    /// Returns a reference to the data contained by the provided id.
    pub fn data_at(&self, id: usize) -> Result<&T, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].data.as_ref().unwrap())
    }

    /// Returns a mutable reference to the data contained by the provided id.
    pub fn data_at_mut(&mut self, id: usize) -> Result<&mut T, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].data.as_mut().unwrap())
    }

    /// Returns the current root of the tree.
    pub fn get_root(&self) -> Option<usize> {
        self.root
    }

    /// Sets the provided data to the new root of the tree removing the old tree.
    pub fn new_root(&mut self, data: T) -> usize{
        if let Some(id) = self.root {
            self.remove(id).unwrap();
        }
        
        let id = self.get_node(data);
        self.root = Some(id);
        id
    }

    /// Set the Node with the given id to the root of the tree, removing the rest of the tree.
    pub fn make_root(&mut self, id: usize) -> Result<(), TreeErr> {
        self.valid_node(id)?;

        self.decouple(id);
        self.remove(self.root.unwrap()).unwrap();
        self.root = Some(id);

        Ok(())
    }

    /// Returns the parent id of the given id.
    pub fn parent_of(&self, id: usize) -> Result<Option<usize>, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].parent)
    }

    /// Returns the next sibling id of the given id.
    pub fn next_sib_of(&self, id: usize) -> Result<Option<usize>, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].next_sib)
    }

    /// Returns the previous sibling id of the given id.
    pub fn prev_sib_of(&self, id: usize) -> Result<Option<usize>, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].prev_sib)
    }

    /// Returns the first child id of the given id.
    pub fn first_child_of(&self, id: usize) -> Result<Option<usize>, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].first_child)
    }

    /// Returns the last child id of the given id.
    pub fn last_child_of(&self, id: usize) -> Result<Option<usize>, TreeErr>{
        self.valid_node(id)?;

        Ok(self.nodes[id].last_child)
    }

    fn valid_move(&self, moving: usize, new_place: usize) -> Result<(), TreeErr> {
        self.valid_node(moving)?;
        self.valid_node(new_place)?;

        for id in self.sub_tree(moving).unwrap() {
            if id == new_place {return Err(CantMoveIntoChild)}
        }

        Ok(())
    }

    /// Moves the given node to be attached to the given node in the position of in_position.
    pub fn move_to(&mut self, moving: usize, in_position: Position, node: usize) -> Result<(), TreeErr> {
        self.valid_move(moving, node)?;

        self.decouple(moving);
        self.attach(moving, in_position, node);

        Ok(())
    }
}

impl<T: Copy + Clone> Tree<T> {
    fn clone_children(&mut self, old_parent: usize, new_parent: usize){
        let mut old_child = self.nodes[old_parent].first_child;

        while let Some(old_child_id) = old_child {
            let new_child = self.get_node(self.nodes[old_child_id].data.unwrap().clone());
            self.append_child(new_parent, new_child);
            self.clone_children(old_child_id, new_child);
            old_child = self.nodes[old_child_id].next_sib;
        }
    }

    fn clone_node(&mut self, id: usize) -> usize {
        let new = self.get_node(self.nodes[id].data.unwrap().clone());
        self.clone_children(id, new);
        new
    }

    /// Clones the given node to be attached to the given node in the position of in_position. Returning the copies new node id.
    pub fn clone_to(&mut self, cloning: usize, in_position: Position, node: usize) -> Result<usize, TreeErr> {
        self.valid_node(cloning)?;
        self.valid_node(node)?;

        let new = self.clone_node(cloning);
        self.attach(new, in_position, node);

        Ok(new)
    }
}

/// The u8 iterator for all of the data in the tree.
pub struct TreeIter<'a, T>{
    tree: &'a Tree<T>,
    nodes_iter: std::vec::IntoIter<NodeInfo>,
    data_iter: Box<dyn std::iter::Iterator<Item = u8> + 'a>,
}

impl<'a, T> TreeIter<'a, T> {
    fn new(tree: &'a Tree<T>) -> Self {
        let nodes = if let Some(root) = tree.get_root(){
            tree.sub_tree_info(root).unwrap()
        }
        else {
            Vec::new()
        };

        let temp = tree.get_root().is_some().into_bytes_static();

        TreeIter {
            data_iter: Box::new(temp),
            nodes_iter: nodes.into_iter(),
            tree: tree,
        }
    }
}

impl<'a, T: IntoBytes<'a>> Iterator for TreeIter<'a, T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item>{
        if let Some(value) = self.data_iter.next(){
            Some(value)
        }
        else {
            if let Some(node) = self.nodes_iter.next(){
                let data = Some(self.tree.data_at(node.id).unwrap());
                let child_count = node.child_count as u32;
                self.data_iter = Box::new(data.unwrap().into_bytes().chain(child_count.into_bytes_static()));
                self.data_iter.next()
            }
            else { None }
        }
    }
}

impl<'a, A: IntoBytes<'a>> IntoBytes<'a> for Tree<A>{

    fn into_bytes(&'a self) -> Box<dyn Iterator<Item = u8> + 'a> {
        Box::new(TreeIter::new(self))
    }
}

impl<A: FromBytes> Tree<A>{
    fn from_bytes_helper<T: Iterator<Item = u8>>(&mut self, parent: usize, bytes: &mut T) -> Result<(), ByteErr>{
        for _ in 0..(u32::from_bytes(bytes)?) as usize {
            let child = self.get_node(A::from_bytes(bytes)?);
            self.append_child(parent, child);
            self.from_bytes_helper(child, bytes)?;
        }

        Ok(())
    }

    fn from_io_bytes_helper<T: Iterator<Item = Result<u8, std::io::Error>>>(&mut self, parent: usize, bytes: &mut T) -> Result<(), ByteErr>{
        for _ in 0..(u32::from_io_bytes(bytes)?) as usize {
            let child = self.get_node(A::from_io_bytes(bytes)?);
            self.append_child(parent, child);
            self.from_io_bytes_helper(child, bytes)?;
        }

        Ok(())
    }
}

impl<A: FromBytes> FromBytes for Tree<A>{
    fn from_bytes<T: Iterator<Item = u8>>(bytes: &mut T) -> Result<Self, ByteErr>{
        
        if bool::from_bytes(bytes)? {
            let mut tree = Tree::new_with_root(A::from_bytes(bytes)?);
            tree.from_bytes_helper(0, bytes)?;
            Ok(tree)
        }
        else {
            Ok(Tree::new())
        }
    }

    fn from_io_bytes<T: Iterator<Item = Result<u8, std::io::Error>>>(bytes: &mut T) -> Result<Self, ByteErr>{

        if bool::from_io_bytes(bytes)? {
            let mut tree = Tree::new_with_root(A::from_io_bytes(bytes)?);
            tree.from_io_bytes_helper(0, bytes)?;
            Ok(tree)
        }
        else {
            Ok(Tree::new())
        }
    }
}