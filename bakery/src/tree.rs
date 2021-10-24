use std::collections::HashMap;

/// Node for `Tree`
#[derive(Debug)]
pub struct TreeItem<T> {
    /// Id of the item in the tree
    /// This is set when the item is created, and must not be changed afterwards.
    pub id: u32,
    /// Id of the parent item in the tree. None if this is a root.
    pub parent: Option<u32>,
    /// Ids of the children items
    pub children: Vec<u32>,
    /// Content of the tree node
    pub value: T,
}

impl<T> TreeItem<T> {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn parent(&self) -> &Option<u32> {
        &self.parent
    }

    pub fn children(&self) -> &Vec<u32> {
        &self.children
    }

    /// Return id of first and only child.
    ///
    /// Panics if there is not exactly on child.
    pub fn unique_child(&self) -> u32 {
        if let Some(id) = self.children.get(0) {
            *id
        } else {
            panic!()
        }
    }

    /// Return id of first and only child, or None.
    ///
    /// Panics if there are more than one child.
    pub fn unique_child_or_none(&self) -> Option<u32> {
        let len = self.children.len();
        assert!(len <= 1);
        self.children.get(0).cloned()
    }
}

/// Tree structure
///
/// Each node in the tree is referenced by a `u32` unique key. The keys are used to reference
/// children and parents in a safe way.
#[derive(Debug)]
pub struct Tree<T> {
    items: HashMap<u32, TreeItem<T>>,
    next_id: u32,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree {
            items: HashMap::new(),
            next_id: 0,
        }
    }

    /// Remove all nodes from the tree. Item id counter is not reset.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Create a new node in the tree and returns its id
    ///
    /// # Arguments
    ///
    /// * `parent` - Id of the parent of the node, None for a root node
    /// * `value` - Value of the node
    pub fn create_with_parent(&mut self, parent: Option<u32>, value: T) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let item = TreeItem {
            id: id,
            parent: parent,
            children: Vec::new(),
            value: value,
        };
        self.items.insert(id, item);
        // If a parent is defined, verify it exists and add the new item as a child
        if let Some(parent) = parent {
            let parent_item = self.items.get_mut(&parent);
            parent_item.unwrap().children.push(id);
        }
        id
    }

    /// Create a new root node in the tree and returns its id
    ///
    /// # Arguments
    ///
    /// * `value` - Value of the node
    pub fn create(&mut self, value: T) -> u32 {
        self.create_with_parent(None, value)
    }

    /// Returns reference to the node of the given id
    ///
    /// # Arguments
    ///
    /// * `id` - Accessed node Id
    pub fn get_item(&self, id: u32) -> &TreeItem<T> {
        self.items.get(&id).unwrap()
    }

    /// Returns mutable reference to the node of the given id
    ///
    /// This method is not public to prevent unproper modification of children or parent.
    ///
    /// # Arguments
    ///
    /// * `id` - Accessed node Id
    fn get_item_mut(&mut self, id: u32) -> &mut TreeItem<T> {
        self.items.get_mut(&id).unwrap()
    }

    /// Returns reference to a node value
    ///
    /// # Arguments
    ///
    /// * `id` - Accessed node Id
    pub fn get(&self, id: u32) -> &T {
        &self.get_item(id).value
    }

    /// Returns mutable reference to a node value
    ///
    /// # Arguments
    ///
    /// * `id` - Accessed node Id
    pub fn get_mut(&mut self, id: u32) -> &mut T {
        &mut self.get_item_mut(id).value
    }

    /// Add a child to a node, and set parent of child node as well
    ///
    /// If the child node already has a parent, the method panics.
    /// If an Id is invalid, the method panics.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Id of the parent node
    /// * `child_id` - Id of the child node
    pub fn child(&mut self, parent_id: u32, child_id: u32) {
        let mut child = self.get_item_mut(child_id);
        assert!(child.parent.is_none());
        child.parent = Some(parent_id);
        let parent = self.get_item_mut(parent_id);
        parent.children.push(child_id);
    }

    /// Returns children of a node
    ///
    /// # Arguments
    ///
    /// * `node` - Parent node Id
    pub fn children(&self, node: u32) -> &Vec<u32> {
        &self.get_item(node).children
    }

    /// Return id of first and only child of a given item.
    ///
    /// Panics if there is not exactly on child.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent item Id.
    pub fn unique_child(&self, parent_id: u32) -> u32 {
        self.get_item(parent_id).unique_child()
    }
}
