

pub struct ShareID {
    pub id: Vec<usize>,
}

impl ShareID {
    pub fn new() -> ShareID {
        ShareID {
            id: vec![0],
        }
    }

    pub fn get_id(&mut self) -> usize {
        match self.id.pop() {
            Some(id) => {
                if self.id.is_empty() {
                    self.id.push(id + 1);
                }
                id
            },
            None => {
                panic!("没有可用的ID");
            }
        }
    }

    pub fn release(&mut self, id: usize) {
        self.id.push(id);
    }
}