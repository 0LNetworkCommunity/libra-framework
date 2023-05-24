/////////////////////////////////////////////////////////////////////////
// 0L Module
// Efficient FIFO Implementation
// Error code for File: 0320
/////////////////////////////////////////////////////////////////////////

/// # Summary
/// Implementation of a FIFO that utilizes two vectors
/// Achieves Amortized O(1) cost per operation 
/// CAUTION: In worst case this can result in O(n) cost, adjust gas allowance accordingly 
module ol_framework::fifo {
    use std::vector;
    use std::error;

    const ACCESSING_EMPTY_FIFO: u64 = 032001;

    /// FIFO implemented using two LIFO vectors 
    /// incoming accepts all pushes (added to the end)
    /// when pop is requested, if outgoing is non-empty, element is popped from the end
    /// if outgoing is empty, all elements are popped from the 
    /// end of incoming and pushed to outoing
    /// result is a FIFO
    struct FIFO<Element> has store, drop {
        incoming: vector<Element>,
        outgoing: vector<Element>
    }

    /// Create an empty FIFO of some type 
    public fun empty<Element>(): FIFO<Element>{
        let incoming = vector::empty<Element>();
        let outgoing = vector::empty<Element>();
        FIFO {
            incoming: incoming,
            outgoing: outgoing,
        }
    }

    /// push an element to the FIFO
    public fun push<Element>(v: &mut FIFO<Element>, new_item: Element){
        vector::push_back<Element>(&mut v.incoming, new_item);
    }

    /// push an element to the end of the FIFO (so it will be popped first) 
    /// useful if you need to give priority to some element
    public fun push_LIFO<Element>(v: &mut FIFO<Element>, new_item: Element){
        vector::push_back<Element>(&mut v.outgoing, new_item);
    }

    /// grab the next element from the queue, removing it
    public fun pop<Element>(v: &mut FIFO<Element>): Element{
        perform_swap<Element>(v);
        //now pop from the outgoing queue
        vector::pop_back<Element>(&mut v.outgoing)
    }

    /// return a ref to the next element in the queue, without removing it 
    public fun peek<Element>(v: &mut FIFO<Element>): & Element{
        perform_swap<Element>(v);

        let len = vector::length<Element>(& v.outgoing);
        vector::borrow<Element>(& v.outgoing, len - 1)
    }

    /// return a mutable ref to the next element in the queue, without removing it 
    public fun peek_mut<Element>(v: &mut FIFO<Element>): &mut Element{
        perform_swap<Element>(v);

        let len = vector::length<Element>(& v.outgoing);
        vector::borrow_mut<Element>(&mut v.outgoing, len - 1)
    }

    /// get the number of elements in the queue
    public fun len<Element>(v: & FIFO<Element>): u64{
        vector::length<Element>(& v.outgoing) + vector::length<Element>(& v.incoming)
    }

    /// internal function, used when peeking and/or popping to move elements 
    /// from the incoming queue to the outgoing queue. Only performs this  
    /// action if the outgoing queue is empty. 
    fun perform_swap<Element>(v: &mut FIFO<Element>) {
        if (vector::length<Element>(& v.outgoing) == 0) {
            let len = vector::length<Element>(&v.incoming);
            assert!(len > 0, error::invalid_state(ACCESSING_EMPTY_FIFO));
            //If outgoing is empty, pop all of incoming into outgoing
            while (len > 0) {
                vector::push_back<Element>(&mut v.outgoing, 
                    vector::pop_back<Element>(&mut v.incoming));
                len = len - 1;
            }
        };
    }

    //// ================ Tests ================

    #[test]
    fun test_basics() {
        let f = empty<u64>();
        let len = len<u64>(& f);
        assert!(len == 0, 1);

        push<u64>(&mut f, 1);
        push<u64>(&mut f, 2);
        push<u64>(&mut f, 3);
        push<u64>(&mut f, 4);
        push<u64>(&mut f, 5);
        push<u64>(&mut f, 6);
        push<u64>(&mut f, 7);

        let len = len<u64>(& f);
        assert!(len == 7, 1);

        let a = pop<u64>(&mut f); //1
        let b = pop<u64>(&mut f); //2
        let c = pop<u64>(&mut f); //3
        let d = pop<u64>(&mut f); //4
        assert!(a == 1, 1);
        assert!(b == 2, 1);
        assert!(c == 3, 1);
        assert!(d == 4, 1);
        let len = len<u64>(& f);
        assert!(len == 3, 1);

        push<u64>(&mut f, 8);
        push<u64>(&mut f, 9);
        push<u64>(&mut f, 10);
        let len = len<u64>(& f);
        assert!(len == 6, 1);

        let e_1 = peek<u64>(&mut f);
        assert!(*e_1 == 5, 1);
        let len = len<u64>(& f);
        assert!(len == 6, 1);
        let e_2 = pop<u64>(&mut f); //5
        assert!(e_2 == 5, 1);
        let len = len<u64>(& f);
        assert!(len == 5, 1);
        
        let f_1 = peek_mut<u64>(&mut f);
        assert!(*f_1 == 6, 1);
        *f_1 = 42;
        
        let len = len<u64>(& f);
        assert!(len == 5, 1);

        let f_2 = pop<u64>(&mut f); //42, changed above
        assert!(f_2 == 42, 1);
        let len = len<u64>(& f);
        assert!(len == 4, 1);

        push_LIFO<u64>(&mut f, 77);
        let g = pop<u64>(&mut f); //77
        assert!(g == 77, 1);

        let g = pop<u64>(&mut f); //7
        let h = pop<u64>(&mut f); //8
        let i = pop<u64>(&mut f); //9
        let j = pop<u64>(&mut f); //10
        assert!(g == 7, 1);
        assert!(h == 8, 1);
        assert!(i == 9, 1);
        assert!(j == 10, 1);  

        let len = len<u64>(& f);
        assert!(len == 0, 1);
    }

}