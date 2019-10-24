pub struct Needle<T> {
    reading: Vec<T>,
    pub index: usize,
    index_stack: Vec<usize>
}

impl Needle<char> {
    pub fn from_str(reading: &str, index: usize) -> Needle<char> {
        Needle {
            reading: reading.chars().collect(),
            index: index,
            index_stack: Vec::new()
        }
    }

    pub fn get_slice(&self, start: usize, stop: usize) -> String {
        let mut string = String::new();

        for i in start..stop {
            string.push(self.reading[i]);
        }

        string
    }

    pub fn matches_slice(&self, slice: &str) -> bool {
        if self.index + slice.len() > self.reading.len() {
            return false;
        }

        unsafe {
            for (i, c) in slice.chars().enumerate() {
                // This is safe because of the check at the top
                if *self.reading.get_unchecked(self.index + i) != c {
                    return false;
                }
            }
        }

        true
    }
}

impl<T> Needle<T> {
    pub fn new(reading: Vec<T>, index: usize) -> Needle<T> {
        Needle {
            reading: reading,
            index: index,
            index_stack: Vec::new()
        }
    }

    pub fn push_state(&mut self) {
        self.index_stack.push(self.index);
    }

    pub fn pop_state(&mut self) {
        assert!(self.index_stack.len() > 0, "Cannot pop_state when index stack length is 0");
        self.index = self.index_stack.pop().unwrap();
    }

    pub fn pop_state_no_revert(&mut self) {
        assert!(self.index_stack.len() > 0, "Cannot pop_state when index stack length is 0");
        self.index_stack.pop();
    }

    pub fn get_prev_state_index(&self) -> usize {
        if let Some(index) = self.index_stack.last() {
            *index
        }else{
            self.index
        }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn peek(&self) -> Option<&T> {
        self.reading.get(self.index)
    }

    pub fn match_func_offset<F>(&self, offset: isize, matching: F) -> bool
            where F: Fn(&T) -> bool {
        let loc = self.index as isize + offset;
        if loc < 0 || loc >= self.reading.len() as isize {
            return false;
        }

        // Safe because of bounds check above
        unsafe {
            matching(self.reading.get_unchecked(loc as usize))
        }
    }

    pub fn next(&mut self) -> bool {
        if self.index >= self.reading.len() { return false; }

        self.index += 1;
        return true;
    }

    pub fn skip(&mut self, n_indices: usize) -> bool {
        self.index += n_indices;
        if self.index >= self.reading.len() {
            self.index = self.reading.len();
            false
        }else{
            true
        }
    }

    pub fn read(&mut self) -> Option<&T> {
        let value = self.reading.get(self.index);
        self.index += 1;
        value
    }
}