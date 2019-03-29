mod tests;

use std::cell::RefCell;

#[derive(Default, Debug)]
struct LinkedListNode<T: Default> {
    next: Option<Box<LinkedListNode<T>>>,
    val: T,
}

impl<T: Default> LinkedListNode<T> {
    pub fn new(val: T) -> LinkedListNode<T> { LinkedListNode{val, next: None} }
}

#[derive(Default, Debug)]
struct LinkedList<T: Default> {
    head: Option<Box<LinkedListNode<T>>>,
    length: usize,
}

impl<'a, T: Default> LinkedList<T> {
    fn new() -> LinkedList<T> { LinkedList::default() }

    fn add(&mut self, val: T) -> () {
        let mut new_node = Box::new(LinkedListNode::new(val));
        std::mem::swap(&mut new_node.next, &mut self.head);
        self.head = Some(new_node);
        self.length += 1;
    }

    fn iter(&'a mut self) -> LinkedListIterator<'a, T> {
        return LinkedListIterator {curr_node: self.head.as_ref() };
    }
}

struct LinkedListIterator<'a, T: Default> {
    curr_node: Option<&'a Box<LinkedListNode<T>>>,
}

impl<'a, T: Default> Iterator for LinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_node.is_none() { return None; }
        let x = self.curr_node.unwrap().next.as_ref();

        std::mem::replace(&mut self.curr_node, x).map(|x| &x.val)
    }
}


fn main() {
    let mut ll = LinkedList::<u32>::new();

    println!("{:#?}", ll);

    ll.add(4);
    ll.add(5);
    ll.add(6);

    println!("{:#?}", ll);

    for i in ll.iter() {
        println!("{}", i);
    }
}
