use crate::analysis::memory::path::Path;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerNullness {
    Null,
    NonNull,
}

#[derive(Clone, Debug, Default)]
pub struct NullnessDomain {
    value_map: HashMap<Rc<Path>, PointerNullness>,
}

impl NullnessDomain {
    pub fn is_empty(&self) -> bool {
        self.value_map.is_empty()
    }

    pub fn get(&self, path: &Rc<Path>) -> Option<PointerNullness> {
        self.value_map.get(path).copied()
    }

    pub fn contains(&self, path: &Rc<Path>) -> bool {
        self.value_map.contains_key(path)
    }

    pub fn set(&mut self, path: Rc<Path>, value: PointerNullness) {
        self.value_map.insert(path, value);
    }

    pub fn forget(&mut self, path: &Rc<Path>) {
        self.value_map.remove(path);
    }

    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if let Some(value) = self.value_map.remove(old_path) {
            self.value_map.insert(new_path.clone(), value);
        } else {
            self.value_map.remove(new_path);
        }
    }

    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if let Some(value) = self.value_map.get(old_path).copied() {
            self.value_map.insert(new_path.clone(), value);
        } else {
            self.value_map.remove(new_path);
        }
    }

    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        self.value_map.keys().cloned().collect()
    }

    pub fn join(&self, other: &Self) -> Self {
        let mut value_map = HashMap::new();
        for path in self.value_map.keys() {
            if let (Some(left), Some(right)) = (self.get(path), other.get(path)) {
                if left == right {
                    value_map.insert(path.clone(), left);
                }
            }
        }
        Self { value_map }
    }

    pub fn meet(&self, other: &Self) -> Self {
        let all_paths: HashSet<Rc<Path>> = self
            .value_map
            .keys()
            .chain(other.value_map.keys())
            .cloned()
            .collect();
        let mut value_map = HashMap::new();
        for path in all_paths {
            match (self.get(&path), other.get(&path)) {
                (Some(left), Some(right)) if left == right => {
                    value_map.insert(path, left);
                }
                (Some(left), None) => {
                    value_map.insert(path, left);
                }
                (None, Some(right)) => {
                    value_map.insert(path, right);
                }
                _ => {}
            }
        }
        Self { value_map }
    }

    pub fn widening_with(&self, other: &Self) -> Self {
        self.join(other)
    }

    pub fn narrowing_with(&self, other: &Self) -> Self {
        self.meet(other)
    }

    pub fn leq(&self, other: &Self) -> bool {
        other
            .value_map
            .iter()
            .all(|(path, value)| self.get(path) == Some(*value))
    }
}
