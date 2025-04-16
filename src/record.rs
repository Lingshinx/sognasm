use std::collections::HashMap;

#[derive(Debug)]
pub struct Record<T>
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
{
    record: HashMap<T, usize>,
    pub data: Vec<T>,
    index: usize,
}

impl<T> Record<T>
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
{
    pub fn new() -> Self {
        Record {
            record: HashMap::new(),
            data: Vec::new(),
            index: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        if let Some(index) = self.record.get(&value) {
            *index
        } else {
            let result = self.index;
            self.record.insert(value, result);
            self.index += 1;
            result
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.data
    }
}
