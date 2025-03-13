use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    nodes: HashSet<String>,
    edges: HashMap<String, HashSet<String>>,
    reverse_edges: HashMap<String, HashSet<String>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: &str) {
        self.nodes.insert(node.to_string());

        if !self.edges.contains_key(node) {
            self.edges.insert(node.to_string(), HashSet::new());
        }

        if !self.reverse_edges.contains_key(node) {
            self.reverse_edges.insert(node.to_string(), HashSet::new());
        }
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        if !self.nodes.contains(from) || !self.nodes.contains(to) {
            return;
        }

        if let Some(edges) = self.edges.get_mut(from) {
            edges.insert(to.to_string());
        }

        if let Some(reverse_edges) = self.reverse_edges.get_mut(to) {
            reverse_edges.insert(from.to_string());
        }
    }

    pub fn get_nodes(&self) -> &HashSet<String> {
        &self.nodes
    }

    pub fn get_dependencies(&self, node: &str) -> Vec<String> {
        if let Some(edges) = self.edges.get(node) {
            return edges.iter().cloned().collect();
        }
        Vec::new()
    }

    pub fn get_dependents(&self, node: &str) -> Vec<String> {
        if let Some(reverse_edges) = self.reverse_edges.get(node) {
            return reverse_edges.iter().cloned().collect();
        }
        Vec::new()
    }

    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for node in &self.nodes {
            if !visited.contains(node) {
                self.find_cycles(node, &mut visited, &mut stack, &mut result);
            }
        }

        result
    }

    fn find_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
        result: &mut Vec<Vec<String>>,
    ) {
        if stack.contains(&node.to_string()) {
            let cycle_start = stack.iter().position(|x| x == node).unwrap();
            let cycle = stack[cycle_start..].to_vec();
            if !result.contains(&cycle) {
                result.push(cycle);
            }
            return;
        }

        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        stack.push(node.to_string());

        if let Some(dependencies) = self.edges.get(node) {
            for dependency in dependencies {
                self.find_cycles(dependency, visited, stack, result);
            }
        }

        stack.pop();
    }

    pub fn to_dot_format(&self) -> String {
        let mut result = String::from("digraph DependencyGraph {\n");
        result.push_str("  node [shape=box, style=filled, fillcolor=lightcyan];\n");
        result.push_str("  edge [color=darkslategray];\n");
        result.push_str("  graph [bgcolor=white];\n");

        for node in &self.nodes {
            let safe_id = node.replace(&['/', '.', '-', ' '][..], "_");
            let short_label = node.split('/').last().unwrap_or(node);
            result.push_str(&format!("  {} [label=\"{}\"];\n", safe_id, short_label));
        }

        for (from, to_set) in &self.edges {
            let from_safe_id = from.replace(&['/', '.', '-', ' '][..], "_");

            for to in to_set {
                let to_safe_id = to.replace(&['/', '.', '-', ' '][..], "_");
                result.push_str(&format!("  {} -> {};\n", from_safe_id, to_safe_id));
            }
        }

        result.push_str("}\n");
        result
    }
}
