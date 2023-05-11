use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub enum VoxelType {
    Grass,
    Dirt
}

const MAX_DEPTH: u32 = 8;

#[derive(Debug)]
pub struct Octree {
    pub root: Node,
    pub depth: u32,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub bitmask: u8,
    pub children: Vec<Box<Node>>,
}

// Basic for loop morton encode
pub fn morton_encode(x: u32, y: u32, z: u32) -> u64 {
    let mut a: u64 = 0;
    for i in 0..21 {
        a |= ((x as u64 & (1 << i)) << 2*i) | ((y as u64 & (1 << i)) << 2*i + 1) | ((z as u64 & (1 << i)) << 2*i + 2)
    }
    return a;
}

pub fn morton_decode(code: u64) -> (u32, u32, u32) {
    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    for i in (0..63).step_by(3).rev() {
        x <<= 1;
        y <<= 1;
        z <<= 1;

        x |= (code & (1 << i)) >> i;
        y |= (code & (1 << (i+1))) >> (i+1);
        z |= (code & (1 << (i+2))) >> (i+2);
    }

    (x as u32, y as u32, z as u32)
}

fn split_by_3(a: u32) -> u64 {
    let x = a & 0x1fffff;
    let mut result = x as u64;
    result |= result << 32;
    result &= 0x1f00000000ffff;
    result |= result << 16;
    result &= 0x1f0000ff0000ff;
    result |= result << 8;
    result &= 0x100f00f00f00f00f;
    result |= result << 4;
    result &= 0x10c30c30c30c30c3;
    result |= result << 2;
    result &= 0x1249249249249249;
    result
}

// Faster magicbits encode
pub fn morton_encode_magicbits(x: u32, y: u32, z: u32) -> u64 {
    split_by_3(x) | split_by_3(y) << 1 | split_by_3(z) << 2
}


impl Node {
    fn new() -> Node {
        Node {
            bitmask: 0,
            children: Vec::new(),
        }
    }

    fn get_child(&self, index: usize) -> Option<&Node> {
        let node_index = self.get_node_index(index);
        if self.children.len() > node_index {
            Some(&self.children[node_index])
        } else {
            None
        }
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut Node> {
        let node_index = self.get_node_index(index);
        if self.children.len() > node_index {
            Some(&mut self.children[node_index])
        } else {
            None
        }
    }

    fn set_child(&mut self, index: usize, child: Node) {
        self.bitmask |= 1 << index;
        let node_index = self.get_node_index(index);
        self.children.insert(node_index, Box::new(child));
    }

    fn get_node_index(&self, index: usize) -> usize {
        let mut bitmask = self.bitmask;
        let mut node_index = 0;
        for i in 0..index {
            node_index += bitmask & 1;
            bitmask >>= 1;
        }
        node_index as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_children() {
        let mut n = Node::new();
        let p = Node::new();
        n.set_child(5, p);
        let q = Node::new();
        n.set_child(1, q);

        let node_index = n.get_node_index(5);
        assert_eq!(node_index, 1);
        let node_index = n.get_node_index(1);
        assert_eq!(node_index, 0);
        let node_index = n.get_node_index(5000);
        assert_eq!(node_index, 2);

        assert_eq!(n.get_child(5).is_none(), false);
        assert_eq!(n.get_child(1).is_none(), false);
        assert_eq!(n.get_child(5000).is_none(), true);
    }
}

impl Octree {
    pub fn new() -> Octree {
        Octree {
            root: Node::new(),
            depth: 0,
        }
    }

    // Returns true if the node exists
    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        let upper_bound = 1 << MAX_DEPTH;
        if x >= upper_bound || y >= upper_bound || z >= upper_bound {
            return false;
        }

        let mut current = &self.root;
        let mut level = 0;
        while level < MAX_DEPTH {
            let index = ((x >> level) & 1) | ((y >> level) & 1) << 1 | ((z >> level) & 1) << 2;
            match current.get_child(index) {
                None => {
                    return false;
                },
                Some(child) => {
                    if child.children.is_empty() {
                        return true;
                    }
                    current = child;
                }
            }

            level += 1;
        }

        return true;
    }

    pub fn insert(&mut self, x: usize, y: usize, z: usize) {
        let mut current = &mut self.root;
        let mut level = 0;
        let mut code = 0;

        while level < MAX_DEPTH {
            let index = ((x >> level) & 1) | ((y >> level) & 1) << 1 | ((z >> level) & 1) << 2;

            let has_child = current.get_child(index).is_some();
            let child;

            if has_child {
                child = current.get_child_mut(index);
            }
            else {
                let node = Node::new();
                current.set_child(index, node);
                child = current.get_child_mut(index);
            }
            current = child.expect("Could not get child!");
            code |= index << (3 * level);
            level += 1;
            if self.depth < level {
                self.depth = level;
            }
        }

        println!("Inserted ({}, {}, {}) with code {:b}", x, y, z, code);
    }
}
