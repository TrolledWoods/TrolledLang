

pub struct TokenError {
    pub msg: &'static str,
    pub loc: usize,
    pub priority: u8
}

impl TokenError {
    pub fn new(loc: usize, priority: u8, msg: &'static str) -> TokenError {
        TokenError {
            loc: loc,
            msg: msg,
            priority: priority
        }
    }

    pub fn at_needle(needle: &StringNeedle, priority: u8, msg: &'static str) -> TokenError {
        TokenError {
            loc: needle.get_index(),
            msg: msg,
            priority: priority
        }
    }

    pub fn if_err_mod<T>(result: Result<T, TokenError>, priority: u8, msg: &'static str) 
            -> Result<T, TokenError> {
        match result {
            Ok(_) => result,
            Err(token_error) => Err(TokenError::new(token_error.loc, priority, msg))
        }
    }
}

pub struct StringNeedle {
    reading: Vec<char>,
    pub index: usize,
    index_stack: Vec<usize>
}

impl StringNeedle {
    pub fn new(reading: &str, index: usize) -> StringNeedle {
        StringNeedle {
            reading: reading.chars().collect(),
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
    
    pub fn peek(&self) -> Option<char> {
        if let Some(c) = self.reading.get(self.index) {
            Some(*c)
        }else{
            None
        }
    }

    pub fn match_func_offset<F>(&self, offset: isize, matching: F, default: bool) -> bool
            where F: Fn(char) -> bool {
        let loc = self.index as isize + offset;
        if loc < 0 || loc >= self.reading.len() as isize {
            return default;
        }

        unsafe {
            matching(*self.reading.get_unchecked(loc as usize))
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

    pub fn read(&mut self) -> Result<char, TokenError> {
        if self.index >= self.reading.len() { return Err(TokenError::new(self.index, 0, "Unexpected end")); }

        self.index += 1;
        Ok(self.reading[self.index - 1])
    }
}