mod byte_vec;
use byte_vec::ByteVec;

pub struct VirtualMachine {
    stack: ByteVec,
    heap: ByteVec
}

impl VirtualMachine {
    pub fn new(instructions: &ByteVec) -> VirtualMachine {
        let mut heap = ByteVec::new();
        heap.push_byte_vec(instructions);
        
        VirtualMachine {
            stack: ByteVec::new(),
            heap: heap
        }
    }
}

pub fn test_vm() {
    let mut bytes = ByteVec::new();
    bytes.push::<Option<i64>>(None);
    bytes.print();
    println!("Number: {}", bytes.peek_from_top::<i64>(8).unwrap());
}