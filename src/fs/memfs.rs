use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

#[derive(Clone)]
pub enum NodeKind {
    File(Vec<u8>),
    Dir(Vec<Box<Node>>),
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub kind: NodeKind,
}

pub struct MemFs {
    root: Node,
}

impl MemFs {
    pub fn new_dir(name: &str) -> Self {
        Self { root: Node { name: name.to_string(), kind: NodeKind::Dir(Vec::new()) } }
    }

    pub fn add_file(&mut self, path: &str, data: &[u8]) {
        let mut parts = path.split('/').filter(|p| !p.is_empty()).peekable();
        let mut cur = &mut self.root;
        while let Some(part) = parts.next() {
            let last = parts.peek().is_none();
            match &mut cur.kind {
                NodeKind::Dir(children) => {
                    if last {
                        children.push(Box::new(Node { name: part.to_string(), kind: NodeKind::File(data.to_vec()) }));
                    } else {
                        let mut found: Option<&mut Node> = None;
                        for ch in children.iter_mut() {
                            if ch.name == part {
                                // SAFETY: limited to this scope
                                let p: *mut Node = ch.as_mut();
                                found = Some(unsafe { &mut *p });
                                break;
                            }
                        }
                        cur = match found {
                            Some(dir @ Node { kind: NodeKind::Dir(_), .. }) => dir,
                            _ => {
                                children.push(Box::new(Node { name: part.to_string(), kind: NodeKind::Dir(Vec::new()) }));
                                children.last_mut().unwrap()
                            }
                        };
                    }
                }
                _ => return,
            }
        }
    }

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, ()> {
        match self.find(path) {
            Some(Node { kind: NodeKind::File(data), .. }) => Ok(data.clone()),
            _ => Err(())
        }
    }

    pub fn list(&self, path: &str) -> Result<Vec<String>, ()> {
        match self.find_ref(path) {
            Some(Node { kind: NodeKind::Dir(children), .. }) => Ok(children.iter().map(|c| c.name.clone()).collect()),
            _ => Err(())
        }
    }

    pub fn write(&mut self, path: &str, data: &[u8]) -> Result<(), ()> {
        // create or overwrite
        let mut parts = path.split('/').filter(|p| !p.is_empty()).peekable();
        let mut cur = &mut self.root;
        while let Some(part) = parts.next() {
            let last = parts.peek().is_none();
            match &mut cur.kind {
                NodeKind::Dir(children) => {
                    if last {
                        // find existing
                        for ch in children.iter_mut() {
                            if ch.name == part {
                                ch.kind = NodeKind::File(data.to_vec());
                                return Ok(());
                            }
                        }
                        children.push(Box::new(Node { name: part.to_string(), kind: NodeKind::File(data.to_vec()) }));
                        return Ok(());
                    } else {
                        let mut next_idx = None;
                        for (i, ch) in children.iter().enumerate() { if ch.name == part { next_idx = Some(i); break; } }
                        if let Some(i) = next_idx { cur = children.get_mut(i).unwrap().as_mut(); }
                        else {
                            children.push(Box::new(Node { name: part.to_string(), kind: NodeKind::Dir(Vec::new()) }));
                            cur = children.last_mut().unwrap();
                        }
                    }
                }
                _ => return Err(()),
            }
        }
        Ok(())
    }

    fn find_ref(&self, path: &str) -> Option<&Node> {
        let mut cur = &self.root;
        for part in path.split('/').filter(|p| !p.is_empty()) {
            match &cur.kind {
                NodeKind::Dir(children) => {
                    let mut next = None;
                    for ch in children.iter() { if ch.name == part { next = Some(ch.as_ref()); break; } }
                    cur = next?;
                }
                _ => return None,
            }
        }
        Some(cur)
    }

    fn find(&mut self, path: &str) -> Option<&mut Node> {
        let mut cur = &mut self.root;
        for part in path.split('/').filter(|p| !p.is_empty()) {
            match &mut cur.kind {
                NodeKind::Dir(children) => {
                    let mut idx = None;
                    for (i, ch) in children.iter().enumerate() { if ch.name == part { idx = Some(i); break; } }
                    cur = children.get_mut(idx?)?.as_mut();
                }
                _ => return None,
            }
        }
        Some(cur)
    }
}
