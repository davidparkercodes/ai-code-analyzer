use ai_code_analyzer::dependency::dependency_graph::DependencyGraph;

#[test]
fn test_dependency_graph_creation() {
    let graph = DependencyGraph::new();
    assert_eq!(graph.get_nodes().len(), 0);
}

#[test]
fn test_adding_nodes_and_edges() {
    let mut graph = DependencyGraph::new();
    
    graph.add_node("node1");
    graph.add_node("node2");
    graph.add_node("node3");
    
    assert_eq!(graph.get_nodes().len(), 3);
    assert!(graph.get_nodes().contains("node1"));
    assert!(graph.get_nodes().contains("node2"));
    assert!(graph.get_nodes().contains("node3"));
    
    graph.add_edge("node1", "node2");
    graph.add_edge("node2", "node3");
    
    assert_eq!(graph.get_dependencies("node1").len(), 1);
    assert!(graph.get_dependencies("node1").contains(&"node2".to_string()));
    
    assert_eq!(graph.get_dependencies("node2").len(), 1);
    assert!(graph.get_dependencies("node2").contains(&"node3".to_string()));
    
    assert_eq!(graph.get_dependencies("node3").len(), 0);
    
    assert_eq!(graph.get_dependents("node1").len(), 0);
    
    assert_eq!(graph.get_dependents("node2").len(), 1);
    assert!(graph.get_dependents("node2").contains(&"node1".to_string()));
    
    assert_eq!(graph.get_dependents("node3").len(), 1);
    assert!(graph.get_dependents("node3").contains(&"node2".to_string()));
}

#[test]
fn test_circular_dependency_detection() {
    let mut graph = DependencyGraph::new();
    
    graph.add_node("A");
    graph.add_node("B");
    graph.add_node("C");
    
    graph.add_edge("A", "B");
    graph.add_edge("B", "C");
    graph.add_edge("C", "A");
    
    let cycles = graph.find_circular_dependencies();
    
    assert_eq!(cycles.len(), 1);
    
    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 3);
    
    let mut graph = DependencyGraph::new();
    
    graph.add_node("A");
    graph.add_node("B");
    graph.add_node("C");
    graph.add_node("D");
    graph.add_node("E");
    
    graph.add_edge("A", "B");
    graph.add_edge("B", "C");
    graph.add_edge("C", "A");
    
    graph.add_edge("D", "E");
    graph.add_edge("E", "D");
    
    let cycles = graph.find_circular_dependencies();
    
    assert_eq!(cycles.len(), 2);
}

#[test]
fn test_dot_format() {
    let mut graph = DependencyGraph::new();
    
    graph.add_node("file1.rs");
    graph.add_node("file2.rs");
    graph.add_edge("file1.rs", "file2.rs");
    
    let dot = graph.to_dot_format();
    
    assert!(dot.starts_with("digraph DependencyGraph {"));
    assert!(dot.ends_with("}\n"));
    
    assert!(dot.contains("file1_rs"));
    assert!(dot.contains("file2_rs"));
    
    assert!(dot.contains("file1_rs -> file2_rs"));
}
