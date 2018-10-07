use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

const OSC: Ordering = Ordering::SeqCst;

#[derive(Debug)]
pub(super) struct Node {
    key: Option<usize>,
    pub val: AtomicPtr<usize>,
    next: AtomicPtr<Node>,
}

impl Node {
    fn new(key: Option<usize>, val: usize) -> Node {
        let v = Box::new(val);
        Node {
            key,
            val: AtomicPtr::new(Box::into_raw(v)),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

#[derive(Debug)]
pub(super) struct LinkedList {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
}

impl LinkedList {
    pub(super) fn new() -> Self {
        let head = Box::new(Node::new(None, 0));
        let tail = Box::into_raw(Box::new(Node::new(None, 0)));
        head.next.store(tail, OSC);

        LinkedList {
            head: AtomicPtr::new(Box::into_raw(head)),
            tail: AtomicPtr::new(tail),
        }
    }

    pub(super) fn insert(
        &self,
        key: usize,
        val: usize,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> Option<*mut usize> {
        let mut new_node = Box::new(Node::new(Some(key), val));
        let mut left_node: *mut Node = ptr::null_mut();

        loop {
            let right_node = self.search(key, &mut left_node, remove_nodes);

            if right_node != self.tail.load(OSC) && unsafe { &*right_node }.key == Some(key) {
                let rn = unsafe { &*right_node };
                let v = Box::new(val);
                let old = rn.val.swap(Box::into_raw(v), OSC);
                return Some(old);
            }

            new_node.next.store(right_node, OSC);

            let new_node_ptr = Box::into_raw(new_node);
            if unsafe { &*left_node }
                .next
                .compare_and_swap(right_node, new_node_ptr, OSC)
                == right_node
            {
                return None;
            }
            new_node = unsafe { Box::from_raw(new_node_ptr) };
        }
    }

    pub(super) fn get(
        &self,
        search_key: usize,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> Option<usize> {
        let mut left_node: *mut Node = ptr::null_mut();
        let right_node = self.search(search_key, &mut left_node, remove_nodes);
        if right_node == self.tail.load(OSC) || unsafe { &*right_node }.key != Some(search_key) {
            None
        } else {
            unsafe { Some(*(&*right_node).val.load(OSC)) }
        }
    }

    pub(super) fn delete(
        &self,
        search_key: usize,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> Option<usize> {
        let mut left_node: *mut Node = ptr::null_mut();
        let mut right_node;
        let mut right_node_next;

        loop {
            right_node = self.search(search_key, &mut left_node, remove_nodes);
            if (right_node == self.tail.load(OSC))
                || unsafe { &*right_node }.key != Some(search_key)
            {
                return None; //failed delete
            }
            right_node_next = unsafe { &*right_node }.next.load(OSC);
            if !Self::is_marked_reference(right_node_next)
                && unsafe { &*right_node }.next.compare_and_swap(
                    right_node_next,
                    Self::get_marked_reference(right_node_next),
                    OSC,
                ) == right_node_next
            {
                break;
            }
        }

        //get value to return
        let rn = unsafe { &*right_node };
        let old = unsafe { *rn.val.load(OSC) };

        if unsafe { &*left_node }
            .next
            .compare_and_swap(right_node, right_node_next, OSC)
            != right_node
        {
            // TODO: do we really not need to do anything with right_node here?
            right_node = self.search(
                unsafe { &*right_node }.key.unwrap(),
                &mut left_node,
                remove_nodes,
            );
        }

        Some(old) //successful delete
    }

    fn is_marked_reference(ptr: *mut Node) -> bool {
        (ptr as usize & 0x1) == 1
    }
    fn get_marked_reference(ptr: *mut Node) -> *mut Node {
        (ptr as usize | 0x1) as *mut Node
    }
    fn get_unmarked_reference(ptr: *mut Node) -> *mut Node {
        (ptr as usize & !0x1) as *mut Node
    }

    fn search(
        &self,
        search_key: usize,
        left_node: &mut *mut Node,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> *mut Node {
        let mut left_node_next: *mut Node = ptr::null_mut();
        let mut right_node;

        //search
        'search_again: loop {
            let mut t = self.head.load(OSC);
            let mut t_next = unsafe { &*t }.next.load(OSC);

            /* 1: Find left_node and right_node */
            loop {
                if !Self::is_marked_reference(t_next) {
                    *left_node = t;
                    left_node_next = t_next;
                }
                t = Self::get_unmarked_reference(t_next);
                if t == self.tail.load(OSC) {
                    break;
                }
                t_next = unsafe { &*t }.next.load(OSC);
                if !Self::is_marked_reference(t_next) && unsafe { &*t }.key >= Some(search_key) {
                    break;
                }
            }
            right_node = t;

            /* 2: Check nodes are adjacent */
            if left_node_next == right_node {
                if right_node != self.tail.load(OSC)
                    && Self::is_marked_reference(unsafe { &*right_node }.next.load(OSC))
                {
                    continue 'search_again;
                } else {
                    return right_node;
                }
            }

            /* 3: Remove one or more marked nodes */
            if unsafe { &**left_node }
                .next
                .compare_and_swap(left_node_next, right_node, OSC)
                == left_node_next
            {
                //drop all of the Nodes that we crossed over,
                //we know nothing inside can be modified so we can just drop all of them with
                //loop until we are at the right_node pointer

                //add to remove_nodes, to be removed
                let mut curr_node = left_node_next; //left_node_next is to be deleted, the ones after it are
                                                    // println!("start curr_node: {:?}", curr_node);

                loop {
                    //start with left_node_next, then go to on until the right_node, but do use that one
                    assert_eq!(Self::is_marked_reference(curr_node), false);
                    remove_nodes.push(curr_node);
                    curr_node = unsafe { &*curr_node }.next.load(OSC);
                    assert_eq!(Self::is_marked_reference(curr_node), true);
                    curr_node = Self::get_unmarked_reference(curr_node); //we need unmarked to deref and comp to right_node
                                                                         // println!("curr_node: {:?}", curr_node);
                    if curr_node == right_node {
                        break;
                    }
                }

                if right_node != self.tail.load(OSC)
                    && Self::is_marked_reference(unsafe { &*right_node }.next.load(OSC))
                {
                    continue 'search_again;
                } else {
                    return right_node;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linkedlist_basics() {
        let mut remove_nodes: Vec<*mut Node> = Vec::new();

        let new_linked_list = LinkedList::new();

        println!("{:?}", new_linked_list);
        new_linked_list.insert(3, 2, &mut remove_nodes);
        new_linked_list.insert(3, 4, &mut remove_nodes);
        new_linked_list.insert(5, 8, &mut remove_nodes);
        new_linked_list.insert(4, 6, &mut remove_nodes);
        new_linked_list.insert(1, 8, &mut remove_nodes);
        new_linked_list.insert(6, 6, &mut remove_nodes);
        //new_linked_list.print();

        assert_eq!(new_linked_list.get(3, &mut remove_nodes).unwrap(), 4);
        assert_eq!(new_linked_list.get(5, &mut remove_nodes).unwrap(), 8);
        assert_eq!(new_linked_list.get(2, &mut remove_nodes), None);
    }

    #[test]
    fn more_linked_list_tests() {
        let mut remove_nodes: Vec<*mut Node> = Vec::new();

        let new_linked_list = LinkedList::new();
        println!(
            "Insert: {:?}",
            new_linked_list.insert(5, 3, &mut remove_nodes)
        );
        println!(
            "Insert: {:?}",
            new_linked_list.insert(5, 8, &mut remove_nodes)
        );
        println!(
            "Insert: {:?}",
            new_linked_list.insert(2, 3, &mut remove_nodes)
        );

        println!("Get: {:?}", new_linked_list.get(5, &mut remove_nodes));

        // println!("{:?}", new_linked_list.head.load(OSC));
        // new_linked_list.print();

        new_linked_list.delete(5, &mut remove_nodes);

        // new_linked_list.print();
    }
}
