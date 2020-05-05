use nano_arena::{Arena, Idx};

struct Connection {
    weight: f32,
    to: Idx,
}

struct Node {
    id: Idx,
    connections: Vec<Connection>,
}

struct Graph {
    arena: Arena<Node>,
}

impl Graph {
    fn add_node(&mut self, target: &Idx) -> Option<&mut Node> {
        let idx = self.arena.alloc_with_idx(|id| Node {
            id,
            connections: vec![],
        });

        if let Some(node) = self.arena.get_mut(target) {
            node.connections.push(Connection {
                to: idx,
                weight: 1.,
            });
            self.arena.get_mut(target)
        } else {
            self.arena.swap_remove(idx);
            None
        }
    }
}

fn main() {}
