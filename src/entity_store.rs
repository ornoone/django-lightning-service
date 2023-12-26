use std::collections::HashMap;
use std::rc::Rc;
use crate::entity::{AttributeDescriptor, Entity, EntityIdentifier, EpochPtr, Model, PK};
use uuid::Uuid;
use crate::errors::EntityError;

struct EntityStore {
    initial_ptr: Rc<EpochPtr>,
    current_ptr: Rc<EpochPtr>,
    entities: EntityStorage,
    index: EntityIdentifierIndex,
}


struct EntityIdentifierIndex {
    entities_pk_index: HashMap<Model, HashMap<PK, Rc<Entity>>>,
    entities_uuid_index: HashMap<Uuid, Rc<Entity>>,
}

impl EntityIdentifierIndex {
    fn new() -> EntityIdentifierIndex {
        EntityIdentifierIndex {
            entities_pk_index: HashMap::new(),
            entities_uuid_index: HashMap::new(),
        }
    }

    /// return the reference to a Entity if the given identifier is already stored
    /// the returned Entity is forcefully the same reference for multiple calls
    fn get(&self, identifier: &EntityIdentifier) -> Result<Rc<Entity>, EntityError> {
        if let Some(result) = self.entities_uuid_index.get(identifier.get_uuid()) {
            return Ok(Rc::clone(result));
        }
        if identifier.has_applied_pk() {
            if let Some(result) = self.entities_pk_index.get(identifier.get_model()).and_then(|hashmap| hashmap.get(identifier.get_applied_pk().unwrap())) {
                return Ok(Rc::clone(result));
            }
        }
        Err(EntityError::EntityNotFound(identifier.clone()))
    }

    fn add(&mut self, entity: Rc<Entity>) {
        let identifier = entity.get_identifier();
        self.entities_uuid_index.insert(identifier.get_uuid().clone(), Rc::clone(&entity));
        if identifier.has_applied_pk() {
            self.entities_pk_index.entry(entity.get_identifier().get_model().clone()).or_insert_with(|| HashMap::new()).insert(identifier.get_applied_pk().unwrap().clone(), Rc::clone(&entity));
        }

    }
}

struct EntityStorage {
    storage: HashMap<Model, Vec<Rc<Entity>>>,
}

impl EntityStorage {
    fn add(&mut self, entity: Entity) -> Rc<Entity> {
        let model = entity.get_identifier().get_model().clone();
        let storage: &mut Vec<Rc<Entity>> = self.storage.entry(model).or_insert(vec![]);
        let rc = Rc::new(entity);
        let result = Rc::clone(&rc);
        storage.push(rc);
        result
    }

    fn new() -> Self {
        EntityStorage {
            storage: HashMap::new(),
        }
    }
}

impl<'a> EntityStore {
    fn get(&self, identifier: &'a EntityIdentifier) -> Result<Rc<Entity>, EntityError> {
        self.index.get(identifier)
    }



    fn add_entity(&'a mut self, entity: Entity) -> Rc<Entity> {
        let identifier = entity.get_identifier();
        let get_result = self.index.get(identifier);
        match get_result {
            Err(EntityError::EntityNotFound(_)) => {
                // add the entity only if it's not already registered
                let res = self.entities.add(entity);
                self.index.add(Rc::clone(&res));
                return res
            },
            Ok(entity) => entity,
            Err(_) => panic!(),
        }
    }

    fn new() -> EntityStore {
        EntityStore {
            initial_ptr: Rc::new(EpochPtr::default()),
            current_ptr: Rc::new(EpochPtr::default()),
            entities: EntityStorage::new(),
            index: EntityIdentifierIndex::new(),
        }
    }

    fn instantiate_entity(&'a mut self, identifier: EntityIdentifier, attributes_descriptors: Vec<AttributeDescriptor>) -> Rc<Entity> {
        let entity = Entity::new(identifier, attributes_descriptors, Rc::clone(&self.initial_ptr), Rc::clone(&self.current_ptr));
        self.add_entity(entity)
    }
}


#[cfg(test)]
mod test {
    use crate::entity::{AttributeDescriptor, AttributeKind, DatabaseValue, EntityIdentifier};
    use crate::entity_store::EntityStore;
    #[test]

    fn test_entity_store_add_and_get() {
        let mut entity_store = EntityStore::new();
        let identifier = EntityIdentifier::new("User".to_string());
        let attributes_descriptors = vec!["name", "age"].iter().map(
            |attr| AttributeDescriptor::new(AttributeKind::Physical, attr.to_string(), DatabaseValue::String(format!("default {}", attr)))
        ).collect();
        let entity = entity_store.instantiate_entity(identifier.clone(), attributes_descriptors);

        let entity_store = entity_store;


        assert_eq!(entity.get("name").unwrap().get_initial(), &DatabaseValue::String("default name".to_string()));
        assert_eq!(entity.get("name").unwrap().get_value(), &DatabaseValue::String("default name".to_string()));

        assert_eq!(entity.get_identifier(), &identifier);
        assert!(entity_store.get(entity.get_identifier()).is_ok());
        assert_eq!(entity_store.get(entity.get_identifier()).unwrap(), entity);
        assert_eq!(&entity_store.get(&identifier).unwrap(), &entity_store.get(&identifier.clone()).unwrap());
    }


}
