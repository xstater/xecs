use crate::{World, StorageId};


#[test]
fn basic() {
    let mut world = World::new();

    world.register::<char>();
    world.register::<()>();
    
    let id1 = world.create_entity()
        .attach('c')
        .into_id();
    let id2 = world.create_entity()
        .attach('b')
        .attach(())
        .into_id();

    {
        let storage = world.storage_read(StorageId::from_rust_type::<char>());
        assert!(storage.is_some());
        let storage = storage.unwrap();
        assert_eq!(storage.len(),2);
        let ch1 = storage.get::<char>(id1).copied();
        assert!(ch1.is_some());
        assert_eq!(ch1.unwrap(),'c');
        let ch2 = storage.get::<char>(id2).copied();
        assert!(ch2.is_some());
        assert_eq!(ch2.unwrap(),'b');
    }

    {
        let storage = world.storage_read(StorageId::from_rust_type::<()>());
        assert!(storage.is_some());
        let storage = storage.unwrap();
        assert_eq!(storage.len(),1);
        let ch1 = storage.get::<()>(id1).copied();
        assert!(ch1.is_none());
        let ch2 = storage.get::<()>(id2).copied();
        assert!(ch2.is_some());
    }
}