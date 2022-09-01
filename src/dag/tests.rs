use crate::dag::DagError;

use super::Dag;

#[test]
fn basic() {
    let mut dag = Dag::new();
    
    dag.insert_node(2, ());
    dag.insert_node(4, ());
    dag.insert_node(3,());
    
    dag.insert_edge(2, 3, ()).unwrap();
    dag.insert_edge(2, 4, ()).unwrap();

    let roots = dag.roots().map(|(id,_)|id).collect::<Vec<_>>();
    let leaves = dag.leaves().map(|(id,_)|id).collect::<Vec<_>>();

    assert_eq!(&roots,&[2]);
    for id in [3,4].iter() {
        assert!(leaves.contains(id))
    }
}

#[test]
fn cycle_test() {
    let mut dag = Dag::new();
    
    dag.insert_node(2, ());
    dag.insert_node(4, ());
    dag.insert_node(3,());
    
    dag.insert_edge(2, 3, ()).unwrap();
    dag.insert_edge(3, 4, ()).unwrap();
    let result = dag.insert_edge(4, 2, ());
    assert!(result.is_err());
    if let DagError::HasCycle(from,to,data) = result.err().unwrap() {
        assert_eq!(from,4);
        assert_eq!(to,2);
        assert_eq!(data,());
    } else {
        unreachable!();
    }


}