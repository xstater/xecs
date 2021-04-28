use crate::EntityId;
use std::num::NonZeroUsize;

pub trait Component : Send + Sync + 'static {}

impl<T : Send + Sync + 'static> Component for T {}

#[derive(Debug,Clone)]
pub(in crate) struct Manager<T : Component>{
    sparse : Vec<Option<NonZeroUsize>>,
    dense : Vec<EntityId>,
    components : Vec<T>
}

impl<T : Component> Manager<T> {
    pub(in crate) fn new() -> Manager<T> {
        Manager {
            sparse: vec![],
            dense: vec![],
            components: vec![]
        }
    }

    pub(in crate) fn len(&self) -> usize {
        self.dense.len()
    }

    pub(in crate) fn new_component(&mut self,entity : EntityId,component : T){
        //enlarge sparse
        while self.sparse.len() <= (entity as usize) {
            self.sparse.push(None);
        }
        if let Some(index) = self.sparse[entity as usize] {
            //already exists
            //overwrite
            self.components[index.get() - 1] = component;
        }else{
            //have not yet
            self.sparse[entity as usize] = NonZeroUsize::new(self.dense.len() + 1);
            self.dense.push(entity);
            self.components.push(component);
        }
    }

    pub(in crate) fn remove_component(&mut self,entity : EntityId) -> Option<T> {
        if self.sparse.len() < (entity as usize) {
            return None;
        }
        if let Some(index) = self.sparse[entity as usize] {
            let index = index.get() - 1;
            self.sparse.swap(self.dense[index] as usize, *self.dense.last().unwrap() as usize);
            self.sparse[entity as usize] = None;
            self.dense.swap_remove(index);
            return Some(self.components.swap_remove(index));
        }
        None
    }

    pub(in crate) fn sparse(&self) -> &[Option<NonZeroUsize>] {
        self.sparse.as_slice()
    }

    pub(in crate) fn sparse_mut(&mut self) -> &mut [Option<NonZeroUsize>] {
        self.sparse.as_mut_slice()
    }

    pub(in crate) fn entities(&self) -> &[EntityId] {
        self.dense.as_slice()
    }

    pub(in crate) fn entities_mut(&mut self) -> &mut [EntityId] {
        self.dense.as_mut_slice()
    }

    pub(in crate) fn components(&self) -> &[T] {
        self.components.as_slice()
    }

    pub(in crate) fn components_mut(&mut self) -> &mut [T] {
        self.components.as_mut_slice()
    }
}

#[cfg(test)]
mod tests{
    use crate::component::Manager;

    #[test]
    fn test(){
        let mut m1 = Manager::new();
        m1.new_component(5,'a');
        m1.new_component(3,'b');
        assert_eq!(m1.entities(),&[5,3]);
        assert_eq!(m1.components(),&['a','b']);
        println!("{:?}",m1);

        m1.new_component(3,'c');
        m1.new_component(1,'d');
        assert_eq!(m1.entities(),&[5,3,1]);
        assert_eq!(m1.components(),&['a','c','d']);
        println!("{:?}",m1);

        assert_eq!(m1.remove_component(2),None);
        assert_eq!(m1.remove_component(5),Some('a'));
        assert_eq!(m1.entities(),&[1,3]);
        assert_eq!(m1.components(),&['d','c']);
        println!("{:?}",m1);
        assert_eq!(m1.remove_component(1),Some('d'));
        assert_eq!(m1.remove_component(3),Some('c'));
        println!("{:?}",m1);
    }
}