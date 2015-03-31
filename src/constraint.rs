//! Constraint graph.

pub trait Lattice {
    type Element;

    fn lub(&self, elem1: Self::Element, elem2: Self::Element) -> Option<Self::Element>;
    fn less_than(&self, elem1: Self::Element, elem2: Self::Element) -> Option<Self::Element>;
}

pub struct ConstraintGraph<L:Lattice> {
    graph: Graph<(),()>,
    values: Vec<L::Element>,
    lattice: L,
}

pub struct Var { pub index: u32 }

impl<L> ConstraintGraph<L>
    where L: Lattice
{
    fn new(lattice: L) -> ConstraintGraph<L> {
        ConstraintGraph {
            graph: Graph::new(),
            values: Vec::new(),
            lattice: L
        }
    }

    fn new_var(&mut self, initial_value: L::Element) -> usize {
        assert_eq!(self.graph.all_nodes().len(), self.values.len());
        let node_index = self.graph.add_node(());
        self.values.push(initial_value);
        Var { index: node_index.0 as u32 }
    }

    fn constrain_var(&mut self,
                     var: Var,
                     value: L::Element)
    {
        self.propagate(var, value);
    }

    fn add_edge(&mut self, source: Var, target: Var) {
        for edge in self.graph.successor_nodes(source) {
        }
    }
}
