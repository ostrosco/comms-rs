use node::Node;

pub struct Graph<'a> {
    nodes: Vec<&'a Node>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Graph<'a>{
        Graph {
            nodes: vec![],
        }
    }

    pub fn add_node(&mut self, node: &'a Node) -> usize {
        self.nodes.push(node);
        self.nodes.len()
    }
}

#[macro_export]
macro_rules! connect_nodes {
    ($graph:ident, $n1:ident, $n2:ident, $recv:ident) => {
        {
            let (recv, send) = crossbeam_channel::unbounded();

        }
    }
}
