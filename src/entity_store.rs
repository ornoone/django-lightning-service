use std::collections::HashMap;
use std::rc::Rc;
use crate::entity::{AttributeDescriptor, Entity, EntityIdentifier, EpochPtr, Model, PK};
use uuid::Uuid;
use crate::errors::EntityError;
use crate::expression::{FilterExpression, match_entity};

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

    fn filter(&self, model: Model, filter_expression: &FilterExpression) -> Result<Vec<Rc<Entity>>, EntityError> {
        if let Some(storage) = self.storage.get(&model) {
            let mut result = vec![];
            for entity in storage {
                if match_entity(filter_expression, entity)? {
                    result.push(Rc::clone(entity))
                }
            }
            Ok(result)
        } else {
            Ok(vec![])
        }
    }
}

impl<'a> EntityStore {
    fn get(&self, identifier: &'a EntityIdentifier) -> Result<Rc<Entity>, EntityError> {
        self.index.get(identifier)
    }

    fn filter(&self, model: Model, filter_expression: &FilterExpression) -> Result<Vec<Rc<Entity>>, EntityError> {

        self.entities.filter(model, filter_expression)
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
    use crate::entity::{AttributeDescriptor, AttributeKind, BaseEntityAttribute, DatabaseValue, EntityAttribute, EntityIdentifier, PhysicalAttribute};
    use crate::entity_store::EntityStore;
    use crate::expression::{ExactExpression, FilterExpression};

    #[test]

    fn test_entity_store_add_and_get() {
        let mut entity_store = EntityStore::new();
        let identifier = EntityIdentifier::new("User".to_string());
        let attributes_descriptors = vec!["name", "age"].iter().map(
            |attr| AttributeDescriptor::new(AttributeKind::Physical, attr.to_string(), DatabaseValue::String(format!("default {}", attr)))
        ).collect();
        let entity = entity_store.instantiate_entity(identifier.clone(), attributes_descriptors);

        let entity_store = entity_store;


        assert_eq!(entity.get("name").unwrap().get_initial(), DatabaseValue::String("default name".to_string()));
        assert_eq!(entity.get("name").unwrap().get_value(), DatabaseValue::String("default name".to_string()));

        assert_eq!(entity.get_identifier(), &identifier);
        assert!(entity_store.get(entity.get_identifier()).is_ok());
        assert_eq!(entity_store.get(entity.get_identifier()).unwrap(), entity);
        assert_eq!(&entity_store.get(&identifier).unwrap(), &entity_store.get(&identifier.clone()).unwrap());
    }

    #[test]
    fn test_entity_identifier_equality() {
        let id1 = EntityIdentifier::new("User".to_string());

        let id2 = EntityIdentifier::new("Book".to_string());
        let id3 = EntityIdentifier::new_persisted("User".to_string(), 1);
        let id4 = EntityIdentifier::new_persisted("User".to_string(), 1);
        let id5 = EntityIdentifier::new_persisted("User".to_string(), 2);

        assert_eq!(id1, id1);
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
        assert_ne!(id1, id5);

        assert_ne!(id2, id1);
        assert_eq!(id2, id2);
        assert_ne!(id2, id3);
        assert_ne!(id2, id4);
        assert_ne!(id2, id5);

        assert_ne!(id3, id1);
        assert_ne!(id3, id2);
        assert_eq!(id3, id3);
        assert_eq!(id3, id4);
        assert_ne!(id3, id5);

        assert_ne!(id4, id1);
        assert_ne!(id4, id2);
        assert_eq!(id4, id3);
        assert_eq!(id4, id4);
        assert_ne!(id4, id5);

        assert_ne!(id5, id1);
        assert_ne!(id5, id2);
        assert_ne!(id5, id3);
        assert_ne!(id5, id4);
        assert_eq!(id5, id5);
    }

    #[test]
    fn test_entity_filter() {

        let mut entity_store = EntityStore::new();
        let attributes_descriptors: Vec<AttributeDescriptor> = vec!["name", "age"].iter().map(
            |attr| AttributeDescriptor::new(AttributeKind::Physical, attr.to_string(), DatabaseValue::String(format!("default {}", attr)))
        ).collect();

        for i in 1..100 {
            let identifier = EntityIdentifier::new("User".to_string());

            let mut entity = entity_store.instantiate_entity(identifier.clone(), attributes_descriptors.clone());
            let mut name_attr = entity.get("name").unwrap();
            name_attr.set_value(DatabaseValue::String(format!("user {}", i)), 1);
            let mut age_attr = entity.get("age").unwrap();
            age_attr.set_value(DatabaseValue::Number(i), 1);
        }

        entity_store.current_ptr.slide(1);
        let list = entity_store.filter("User".to_string(), &FilterExpression::Exact(ExactExpression::new("name".to_string(), DatabaseValue::String("user 4".to_string())))).unwrap();
        assert_eq!(list.len(), 1);
        let entity = list.first().unwrap();
        assert_eq!(entity.get("name").unwrap().get_value(), DatabaseValue::String("user 4".to_string()));

    }


}
