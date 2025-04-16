use std::collections::HashMap;

#[derive(Debug)]
pub struct Record<T>
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
{
    record: HashMap<T, usize>,
    pub data: Vec<T>,
}

impl<T> Record<T>
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
    T: Clone,
{
    pub fn new() -> Self {
        Record {
            record: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        if let Some(index) = self.record.get(&value) {
            *index
        } else {
            let index = self.data.len();
            self.record.insert(value.clone(), index);
            self.data.push(value);
            index
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.data
    }
}
