use std::num::NonZeroUsize;

#[derive(Debug,Clone)]
pub(in crate) struct SparseSet<E,T>
    where E : Copy + Into<usize>,
          T : Sized{
    pub (in crate) indices : Vec<Option<NonZeroUsize>>,
    pub (in crate) entities :  Vec<E>,
    pub (in crate) data : Vec<T>
}

impl<E,T> SparseSet<E,T>
    where E : Copy + Into<usize>,
          T : Sized {

    pub fn new() -> Self {
        SparseSet{
            indices: vec![],
            entities: vec![],
            data: vec![]
        }
    }

    pub fn clear(&mut self){
        self.indices.clear();
        self.entities.clear();
        self.data.clear();
    }

    pub fn add(&mut self,entity : E,data : T) {
        let entity_ : usize = entity.into();
        //enlarge sparse
        while self.indices.len() <= entity_ {
            self.indices.push(None);
        }
        if let Some(index) = self.indices[entity_] {
            //already exists
            //overwrite
            self.data[index.get() - 1] = data;
        }else{
            //not yet exist
            self.indices[entity_] = NonZeroUsize::new(self.entities.len() + 1);
            self.entities.push(entity);
            self.data.push(data);
        }
    }

    pub fn add_batch(&mut self,entities : &[E],mut data : Vec<T>) {
        assert_eq!(entities.len(),data.len());
        let start_index = self.entities.len();
        // copy data to dense
        self.entities.extend_from_slice(entities);
        self.data.append(&mut data);
        // store data in sparse
        for (index,entity) in entities.iter().enumerate() {
            let entity_ : usize = (*entity).into();
            // enlarge sparse
            while self.indices.len() <= entity_ {
                self.indices.push(None);
            }
            // store index to sparse
            self.indices[entity_] = Some(unsafe {
                NonZeroUsize::new_unchecked(start_index + index + 1)
            });
        }
    }

    pub fn remove(&mut self,entity : E) -> Option<T> {
        let entity : usize = entity.into();
        if self.indices.len() < entity {
            return None;
        }
        if let Some(index) = self.indices[entity] {
            let index = index.get() - 1;
            self.indices.swap(self.entities[index].into(),(*self.entities.last().unwrap()).into());
            self.indices[entity] = None;
            self.entities.swap_remove(index);
            return Some(self.data.swap_remove(index));
        }
        None
    }

    pub(in crate) fn swap_by_index(&mut self,index_a : usize,index_b : usize) {
        if index_a == index_b { return; }
        if index_a >= self.len() {
            panic!("index_a={} is out of range",index_a);
        }
        if index_b >= self.len() {
            panic!("index_b={} is out of range",index_b);
        }
        let entity_a : usize = self.entities[index_a].into();
        let entity_b : usize = self.entities[index_b].into();
        self.indices.swap(entity_a,entity_b);
        self.entities.swap(index_a,index_b);
        self.data.swap(index_a,index_b);
    }

    #[allow(unused)]
    pub(in crate) fn swap_by_entity(&mut self,entity_a : E,entity_b : E) {
        if !self.exist(entity_a) {
            panic!("entity_a is not exist in sparse set");
        }
        if !self.exist(entity_b) {
            panic!("entity_b is not exist in sparse set");
        }
        let entity_a : usize = entity_a.into();
        let entity_b : usize = entity_b.into();
        if entity_a == entity_b { return; }
        let index_a = self.indices[entity_a].unwrap().get() - 1;
        let index_b = self.indices[entity_b].unwrap().get() - 1;
        self.indices.swap(entity_a,entity_b);
        self.entities.swap(index_a,index_b);
        self.data.swap(index_a,index_b);
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn exist(&self,entity : E) -> bool {
        let entity : usize = entity.into();
        if entity < self.indices.len()  {
            self.indices[entity].is_some()
        }else{
            false
        }
    }

    pub fn get(&self,entity : E) -> Option<&T> {
        let entity : usize = entity.into();
        if entity< self.indices.len() {
            if let Some(index) = self.indices[entity] {
                let index = index.get() - 1;
                return Some(&self.data[index])
            }
        }
        None
    }

    pub unsafe fn get_unchecked(&self,entity : E) -> &T {
        let entity : usize = entity.into();
        let index = self.indices.get_unchecked(entity).unwrap().get();
        self.data.get_unchecked(index - 1)
    }

    pub fn get_mut(&mut self,entity : E) -> Option<&mut T> {
        let entity : usize = entity.into();
        if entity < self.indices.len() {
            if let Some(index) = self.indices[entity] {
                let index = index.get() - 1;
                return Some(&mut self.data[index])
            }
        }
        None
    }

    pub unsafe fn get_unchecked_mut(&mut self,entity : E) -> &mut T {
        let entity : usize = entity.into();
        let index = self.indices.get_unchecked(entity).unwrap().get();
        self.data.get_unchecked_mut(index - 1)
    }

    pub fn get_index(&self,entity : E) -> Option<usize> {
        let entity : usize = entity.into();
        if entity < self.indices.len() {
            if let Some(index) = self.indices[entity] {
                return Some(index.get() - 1);
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.entities.len() == 0
    }

    #[allow(unused)]
    pub fn indices(&self) -> &[Option<NonZeroUsize>] {
        self.indices.as_slice()
    }

    pub fn entities(&self) -> &[E] {
        self.entities.as_slice()
    }

    #[allow(unused)]
    pub fn entities_mut(&mut self) -> &mut [E] {
        self.entities.as_mut_slice()
    }

    pub fn data(&self) -> &[T] {
        self.data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }
}

#[cfg(test)]
mod tests{
    use crate::sparse_set::SparseSet;

    #[test]
    fn basic_test(){
        let mut s1 = SparseSet::new();
        s1.add(5usize,'a');
        s1.add(3,'b');
        assert_eq!(s1.entities(),&[5,3]);
        assert_eq!(s1.data(),&['a','b']);
        println!("{:?}",s1);

        s1.add(3,'c');
        s1.add(1,'d');
        assert_eq!(s1.entities(),&[5,3,1]);
        assert_eq!(s1.data(),&['a','c','d']);
        println!("{:?}",s1);

        assert_eq!(s1.get(4),None);
        assert_eq!(s1.get(1),Some(&'d'));
        *s1.get_mut(1).unwrap() = 'f';
        assert_eq!(s1.get(1),Some(&'f'));
        assert_eq!(s1.get_index(3),Some(1));
        println!("{:?}",s1);
        *s1.get_mut(1).unwrap() = 'd';

        assert_eq!(s1.remove(2),None);
        assert_eq!(s1.remove(5),Some('a'));
        assert_eq!(s1.entities(),&[1,3]);
        assert_eq!(s1.data(),&['d','c']);
        println!("{:?}",s1);
        assert_eq!(s1.remove(1),Some('d'));
        assert_eq!(s1.remove(3),Some('c'));
        println!("{:?}",s1);
        assert!(s1.is_empty());
    }

    #[test]
    fn swap_test(){
        let mut s1 = SparseSet::new();
        s1.add(3usize,'a');
        s1.add(5,'b');
        s1.add(6,'c');
        s1.add(2,'d');
        assert_eq!(s1.entities(),&[3,5,6,2]);
        assert_eq!(s1.data(),&['a','b','c','d']);
        println!("{:?}",s1);

        s1.swap_by_index(1,2);
        assert_eq!(s1.entities(),&[3,6,5,2]);
        assert_eq!(s1.data(),&['a','c','b','d']);
        println!("{:?}",s1);

        s1.swap_by_entity(2,3);
        assert_eq!(s1.entities(),&[2,6,5,3]);
        assert_eq!(s1.data(),&['d','c','b','a']);
        println!("{:?}",s1);
    }

    #[test]
    fn batch() {
        let mut s = SparseSet::new();
        let entities = [2_usize,5,3,4];
        let data = vec!['a','b','c','d'];
        s.add_batch(&entities,data);
        println!("{:?}",s);

        let entities = [1_usize,6];
        let data = vec!['e','f'];
        s.add_batch(&entities,data);
        println!("{:?}",s);
    }
}
