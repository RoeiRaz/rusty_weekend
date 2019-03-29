use crate::{LinkedListIterator, LinkedList};

#[test]
fn test_linked_list_empty() {
    let mut ll = LinkedList::<u32>::new();

    let mut it = ll.iter();

    assert_eq!(it.next(), None);
}

#[test]
fn test_linked_list_some_iterator() {
    let mut ll = LinkedList::new();
    ll.add(1);
    ll.add(2);
    ll.add(3);

    let mut it = ll.iter();

    assert_eq!(it.next(), Some(&3));
    assert_eq!(it.next(), Some(&2));
    assert_eq!(it.next(), Some(&1));
    assert_eq!(it.next(), None);
}