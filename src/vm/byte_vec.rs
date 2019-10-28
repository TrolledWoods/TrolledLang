use std::mem::size_of;

const BATCH_SIZE_BYTES: usize = 8;
const BATCH_SIZE: usize = 1 << BATCH_SIZE_BYTES;

pub struct ByteVecIterator<'a> {
    byte_vec: &'a ByteVec,
    batch_index: usize,
    local_index: usize
}

impl<'a> Iterator for ByteVecIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.batch_index <= self.byte_vec.batch_index && self.local_index <= self.byte_vec.local_index {
            let result = unsafe {
                let batch = self.byte_vec.batches.get_unchecked(self.batch_index);
                batch.get_unchecked(self.local_index)
            };

            self.local_index += 1;
            if self.local_index >= BATCH_SIZE {
                self.local_index -= BATCH_SIZE;
                self.batch_index += 1;
            }

            Some(*result)
        }else{
            None
        }
    }
}

pub struct ByteVec {
    batches: Vec<[u8; 1 << BATCH_SIZE_BYTES]>,
    local_index: usize,
    batch_index: usize
}

impl ByteVec {
    pub fn new() -> ByteVec {
        ByteVec {
            batches: vec![[0; BATCH_SIZE]],
            local_index: 0,
            batch_index: 0
        }
    }

    pub fn get_index(&self) -> usize {
        (self.batch_index << BATCH_SIZE_BYTES) + self.local_index
    }

    pub fn iter(&self) -> ByteVecIterator {
        ByteVecIterator {
            byte_vec: self,
            batch_index: 0,
            local_index: 0
        }
    }

    unsafe fn get_byte_unchecked(&self, index: usize) -> u8 {
        let batch_index = index >> BATCH_SIZE_BYTES;
        let batch = self.batches[batch_index];
        batch[index - (batch_index << BATCH_SIZE_BYTES)]
    }

    unsafe fn get_value_slow_unchecked<T: Sized + Clone + std::fmt::Display>(&self, index: usize) -> T {
        let temp_buf = &mut [0u8; 32];
        for i in 0..size_of::<T>() {
            temp_buf[i] = self.get_byte_unchecked(index + i);
        }

        let value = temp_buf.as_ptr();
        let ptr = value as *const T;
        let cl = (*ptr).clone();
        cl
    }

    pub fn print(&self) {
        print!("bytes:");
        for i in 0..self.get_index() {
            unsafe {
                print!(" {:x}", self.get_byte_unchecked(i));
            }
        }
        println!("");
    }

    pub fn peek_from_top<T: Sized + Clone + std::fmt::Display>(&self, back_offset: usize) -> Option<T> {
        let index = self.get_index();
        if back_offset > index || back_offset < size_of::<T>() {
            println!("Backoffset: {}, index: {}, size: {}", back_offset, index, size_of::<T>());
            return None;
        }

        let index = index - back_offset;
        let batch_index = index >> BATCH_SIZE_BYTES;
        let local_index = index - (batch_index << BATCH_SIZE_BYTES);

        // Check if we're outside the current batch
        if local_index + size_of::<T>() >= BATCH_SIZE {
            unsafe {
                Some(self.get_value_slow_unchecked::<T>(index))
            }
        }else{
            unsafe {
                // get_unchecked is safe because we checked the bounds above
                let batch = self.batches.get_unchecked(batch_index);
                let value = batch.as_ptr().offset(local_index as isize);
                // This is safe because we clone it, we don't use another reference.
                // However, the type could still be improperly formatted in memory,
                // so naturally it's not quite safe.
                // But machine code is not considered safe anyway,
                // so I don't really care xD
                let other = value as *const T;
                Some((*other).clone())
            }
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.batches[self.batch_index][self.local_index] = byte;
        
        self.local_index += 1;
        while self.local_index >= BATCH_SIZE {
            self.local_index -= BATCH_SIZE;
            self.batch_index += 1;
            self.batches.push([0; BATCH_SIZE]);
        }
    }

    pub fn push_byte_vec(&mut self, byte_vec: &ByteVec) {
        for byte in byte_vec.iter() {
            self.push_byte(byte);
        }
    }
    
    pub fn push<T: Sized>(&mut self, value: T) {
        let t_size = size_of::<T>();

        unsafe {
            let ptr = (&value as *const T) as *const u8;
            for i in 0..t_size {
                self.push_byte(*ptr.offset(i as isize));
            }
        }
    }
}