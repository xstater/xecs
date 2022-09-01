use crate::dag::DagError;

use super::Dag;

#[test]
fn basic() {
    let mut dag = Dag::new();

    dag.insert_node(2, ());
    dag.insert_node(4, ());
    dag.insert_node(3, ());

    dag.insert_edge(2, 3, ()).unwrap();
    dag.insert_edge(2, 4, ()).unwrap();

    let roots = dag.roots().map(|(id, _)| id).collect::<Vec<_>>();
    let leaves = dag.leaves().map(|(id, _)| id).collect::<Vec<_>>();

    assert_eq!(&roots, &[2]);
    for id in [3, 4].iter() {
        assert!(leaves.contains(id))
    }
}

#[test]
fn cycle_test() {
    let mut dag = Dag::new();

    dag.insert_node(2, ());
    dag.insert_node(4, ());
    dag.insert_node(3, ());

    dag.insert_edge(2, 3, ()).unwrap();
    dag.insert_edge(3, 4, ()).unwrap();
    let result = dag.insert_edge(4, 2, ());
    assert!(result.is_err());
    if let DagError::HasCycle(from, to, data) = result.err().unwrap() {
        assert_eq!(from, 4);
        assert_eq!(to, 2);
        assert_eq!(data, ());
    } else {
        unreachable!();
    }
}

#[test]
fn insert_and_remove() {
    let mut dag = Dag::new();

    // build a dag
    dag.insert_node(1, 'A');
    dag.insert_node(2, 'B');
    dag.insert_node(3, 'C');
    dag.insert_node(4, 'D');
    dag.insert_node(5, 'E');

    dag.insert_edge(1, 2, 'a').unwrap();
    dag.insert_edge(2, 3, 'b').unwrap();
    dag.insert_edge(4, 2, 'c').unwrap();
    dag.insert_edge(4, 5, 'd').unwrap();
    dag.insert_edge(3, 5, 'e').unwrap();
    let result = dag.insert_edge(3, 4, 'f');
    // it must fail and dag will not be destoryed
    assert!(result.is_err());
    if let DagError::HasCycle(from, to, data) = result.unwrap_err() {
        assert_eq!(from, 3);
        assert_eq!(to, 4);
        assert_eq!(data, 'f');
    } else {
        unreachable!()
    }

    // check dag
    {
        // root
        let mut roots = dag.roots().map(|(id, _)| id).collect::<Vec<_>>();
        roots.sort();
        assert_eq!(&roots, &[1, 4]);
        // leaves
        let mut leaves = dag.leaves().map(|(id, _)| id).collect::<Vec<_>>();
        leaves.sort();
        assert_eq!(&leaves, &[5]);
        // all nodes
        let mut children = dag.children(1).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[2]);

        let mut children = dag.children(2).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[3]);

        let mut children = dag.children(3).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[5]);

        let mut children = dag.children(4).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[2, 5]);

        let mut children = dag.children(5).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        // parents
        let mut parents = dag.parents(1).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(2).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[1, 4]);

        let mut parents = dag.parents(3).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[2]);

        let mut parents = dag.parents(4).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(5).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[3, 4]);

        // data stored in node
        let node_data = (1..=5)
            .map(|node| dag.get_node(node).unwrap())
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(&node_data, &['A', 'B', 'C', 'D', 'E']);
        // edge_data test
        let mut edges_data = dag
            .edges()
            .map(|(_, _, data)| data)
            .copied()
            .collect::<Vec<_>>();
        edges_data.sort();
        assert_eq!(&edges_data, &['a', 'b', 'c', 'd', 'e']);
    }

    let result = dag.remove_edge(3, 5);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 'e');

    {
        // root
        let mut roots = dag.roots().map(|(id, _)| id).collect::<Vec<_>>();
        roots.sort();
        assert_eq!(&roots, &[1, 4]);
        // leaves
        let mut leaves = dag.leaves().map(|(id, _)| id).collect::<Vec<_>>();
        leaves.sort();
        assert_eq!(&leaves, &[3, 5]);
        // all nodes
        let mut children = dag.children(1).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[2]);

        let mut children = dag.children(2).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[3]);

        let mut children = dag.children(3).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        let mut children = dag.children(4).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[2, 5]);

        let mut children = dag.children(5).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        // parents
        let mut parents = dag.parents(1).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(2).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[1, 4]);

        let mut parents = dag.parents(3).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[2]);

        let mut parents = dag.parents(4).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(5).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[4]);

        // data stored in node
        let node_data = (1..=5)
            .map(|node| dag.get_node(node).unwrap())
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(&node_data, &['A', 'B', 'C', 'D', 'E']);
        // edge_data test
        let mut edges_data = dag
            .edges()
            .map(|(_, _, data)| data)
            .copied()
            .collect::<Vec<_>>();
        edges_data.sort();
        assert_eq!(&edges_data, &['a', 'b', 'c', 'd']);
    }

    let (node_data, mut edges_data) = dag.remove_node(2);
    assert!(node_data.is_some());
    assert_eq!(node_data.unwrap(), 'B');
    edges_data.sort();
    assert_eq!(&edges_data, &['a', 'b', 'c']);
    {
        // root
        let mut roots = dag.roots().map(|(id, _)| id).collect::<Vec<_>>();
        roots.sort();
        assert_eq!(&roots, &[1, 3, 4]);
        // leaves
        let mut leaves = dag.leaves().map(|(id, _)| id).collect::<Vec<_>>();
        leaves.sort();
        assert_eq!(&leaves, &[1, 3, 5]);
        // all nodes
        let mut children = dag.children(1).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        let mut children = dag.children(2).map(|(id, _)| id).collect::<Vec<_>>();
        // when id desn't exist in dag, it yield None
        children.sort();
        assert_eq!(&children, &[]);

        let mut children = dag.children(3).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        let mut children = dag.children(4).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[5]);

        let mut children = dag.children(5).map(|(id, _)| id).collect::<Vec<_>>();
        children.sort();
        assert_eq!(&children, &[]);

        // parents
        let mut parents = dag.parents(1).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(2).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(3).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(4).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[]);

        let mut parents = dag.parents(5).collect::<Vec<_>>();
        parents.sort();
        assert_eq!(&parents, &[4]);

        // data stored in node
        let node_data = (1..=5)
            .map(|node| dag.get_node(node))
            .filter(|node| node.is_some())
            .map(|node| node.unwrap())
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(&node_data, &['A', 'C', 'D', 'E']);
        // edge_data test
        let mut edges_data = dag
            .edges()
            .map(|(_, _, data)| data)
            .copied()
            .collect::<Vec<_>>();
        edges_data.sort();
        assert_eq!(&edges_data, &['d']);
    }
}
