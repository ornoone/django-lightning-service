use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use uuid::Uuid;
use crate::errors::EntityError;

pub type Epoch = i64;
pub type Model = String;

#[derive(Debug)]
pub struct EpochPtr {
    epoch: RefCell<Epoch>,
}

impl Default for EpochPtr {
    fn default() -> Self {
        EpochPtr { epoch: RefCell::new(0) }
    }
}

impl EpochPtr {
    pub fn slide(&self, epoch: Epoch) {
        let mut ep = self.epoch.borrow_mut();
        *ep = epoch;
    }

    pub fn get_epoch(&self) -> Epoch {
        *self.epoch.borrow()
    }
}

#[derive(Debug)]
struct AttributeValue<T> {
    epoch: Epoch,
    value: T,
}

#[derive(Debug, Clone)]

pub enum DatabaseValue {
    String(String),
    Number(i64),
    None,
}

impl PartialEq for DatabaseValue {
    fn eq(&self, other: &Self) -> bool {
        use DatabaseValue::*;

        match (self, other) {
            (None, None) => true,
            (None, _) => false,
            (_, None) => false,
            (&String(ref a), &String(ref b)) => a == b,
            (&Number(ref a), &Number(ref b)) => a == b,
            _ => false,
        }
    }
}

pub trait BaseEntityAttribute {
    fn get_initial(&self) -> DatabaseValue;

    fn get_value(&self) -> DatabaseValue;

    fn set_value(&self, value: DatabaseValue, epoch: Epoch);
}

pub trait EntityAttribute: Debug + BaseEntityAttribute {}

impl<T: Debug + BaseEntityAttribute> EntityAttribute for T {}

#[derive(Debug)]
pub struct PhysicalAttribute {
    current_epoch_ptr: Rc<EpochPtr>,

    initial_epoch_ptr: Rc<EpochPtr>,

    value_history: RefCell<Vec<AttributeValue<DatabaseValue>>>,
}

impl PhysicalAttribute {
    fn new(current_epoch_ptr: Rc<EpochPtr>, initial_epoch_ptr: Rc<EpochPtr>) -> Self {
        PhysicalAttribute {
            current_epoch_ptr,
            initial_epoch_ptr,
            value_history: RefCell::new(vec!()),
        }
    }


    fn get_at_epoch(&self, epoch: Epoch) -> DatabaseValue {
        let value_history = self.value_history.borrow();
        for history in value_history.iter().rev() {
            if history.epoch <= epoch {
                return history.value.clone();
            }
        }
        // return the initial value instead
        let initial = value_history.first().unwrap();
        return initial.value.clone();
    }

    fn insert_at_epoch(&self, value: DatabaseValue, epoch: Epoch) {
        let mut value_history = self.value_history.borrow_mut();

        let history_value = AttributeValue { epoch, value };
        for (i, hist) in value_history.iter().enumerate() {
            if hist.epoch > epoch {
                value_history.insert(i, history_value);
                return;
            }
        }

        value_history.push(history_value);
    }
}


impl<'a> BaseEntityAttribute for PhysicalAttribute {
    fn get_initial(&self) -> DatabaseValue {
        self.get_at_epoch(self.initial_epoch_ptr.get_epoch())
    }

    fn get_value(&self) -> DatabaseValue {
        self.get_at_epoch(self.current_epoch_ptr.get_epoch())
    }

    fn set_value(&self, value: DatabaseValue, epoch: Epoch) {
        self.insert_at_epoch(value, epoch);
    }
}

pub type PK = i64;

#[derive(Debug)]
#[derive(Clone)]
pub struct EntityIdentifier {
    model: Model,
    pk: Option<PK>,
    uuid: Uuid
}

impl PartialEq for EntityIdentifier {
    fn eq(&self, other: &Self) -> bool {
        if self.uuid == other.uuid {
            return true
        }
        self.has_applied_pk() && other.has_applied_pk() && self.model == other.model && self.pk == other.pk
    }
}

impl EntityIdentifier {
    pub fn new(model: Model) -> EntityIdentifier {
        EntityIdentifier {
            model,
            pk: None,
            uuid: Uuid::new_v4()
        }
    }

    pub fn new_persisted(model: Model, pk: PK) -> EntityIdentifier {
        EntityIdentifier {
            model,
            pk: Some(pk),
            uuid: Uuid::new_v4()
        }
    }


    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn get_model(&self) -> &Model {
        &self.model
    }

    pub fn has_applied_pk(&self) -> bool {
        self.pk.is_some()
    }

    pub fn get_applied_pk(&self) -> Result<&PK, EntityError> {
        match &self.pk {
            None => Err(EntityError::UnpersistedEntity(self.clone())),
            Some(pk) => Ok(pk)
        }

    }

    pub fn set_applied_pk(&mut self, pk: PK) {
        self.pk = Some(pk)
    }
}


#[derive(Debug)]
pub struct Entity {
    identifier: EntityIdentifier,
    physical_attributes: HashMap<String, PhysicalAttribute>,
}

#[derive(Clone, Debug)]
pub enum AttributeKind {
    Physical,
    ManyToMany,
}

#[derive(Clone, Debug)]
pub struct AttributeDescriptor {
    kind: AttributeKind,
    name: String,
    initial: DatabaseValue,
}

impl AttributeDescriptor {
    pub fn new(kind: AttributeKind, name: String, initial: DatabaseValue) -> Self {
        AttributeDescriptor {
            kind,
            name,
            initial
        }
    }
}

impl<'a> PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }

    fn ne(&self, other: &Self) -> bool {
        self.identifier != other.identifier
    }
} 

impl<'a> Entity {
    pub fn new(identifier: EntityIdentifier, attributes: Vec<AttributeDescriptor>, initial_ptr: Rc<EpochPtr>, current_ptr: Rc<EpochPtr>) -> Self {
        let mut physicals: HashMap<String, PhysicalAttribute> = HashMap::new();

        for attribute in attributes {
            match attribute.kind {
                AttributeKind::ManyToMany => panic!("not yet implemented"),
                AttributeKind::Physical => {
                    let mut attr = PhysicalAttribute::new(Rc::clone(&current_ptr), Rc::clone(&initial_ptr));
                    attr.set_value(attribute.initial, initial_ptr.get_epoch());
                    physicals.insert(attribute.name, attr);
                }
            }
        }
        Entity {
            identifier,
            physical_attributes: physicals,
        }
    }

    pub fn get<'b>(&'a self, attribute: &'b str) -> Result<&'a (dyn EntityAttribute), EntityError> {
        if let Some(attr) = self.physical_attributes.get(attribute) {
            Ok(attr)
        } else {
            Err(EntityError::AttributeNotFound(attribute.to_string()))
        }
    }

    pub fn get_identifier(&'a self) -> &EntityIdentifier {
        &self.identifier
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::entity::{AttributeDescriptor, AttributeKind, BaseEntityAttribute, DatabaseValue, Entity, EntityIdentifier, EpochPtr, PhysicalAttribute};
    use crate::errors::EntityError;

    #[test]
    fn get_ptr_slide() {
        let initial_ptr = Rc::new(EpochPtr::default());
        let current_ptr = Rc::new(EpochPtr::default());
        current_ptr.slide(2);
        let mut attr: PhysicalAttribute = PhysicalAttribute::new(Rc::clone(&current_ptr), initial_ptr);
        attr.set_value(DatabaseValue::Number(42), 0);
        attr.set_value(DatabaseValue::Number(52), 2);

        assert_eq!(attr.get_initial(), DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), DatabaseValue::Number(52));
        current_ptr.slide(3);
        assert_eq!(attr.get_initial(), DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), DatabaseValue::Number(52));

        current_ptr.slide(0);
        assert_eq!(attr.get_initial(), DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), DatabaseValue::Number(42));
    }

    #[test]
    fn test_entity() {
        let initial_ptr = Rc::new(EpochPtr::default());
        let current_ptr = Rc::new(EpochPtr::default());
        let entity = Entity::new(
            EntityIdentifier::new("User".to_string()),
            vec![AttributeDescriptor { kind: AttributeKind::Physical, name: String::from("name"), initial: DatabaseValue::String("john".to_string()) }],
            initial_ptr,
            current_ptr,
        );

        assert_eq!(entity.get("name").unwrap().get_initial(), DatabaseValue::String("john".to_string()))
    }


    #[test]
    fn test_attr_not_found() {
        let initial_ptr = Rc::new(EpochPtr::default());
        let current_ptr = Rc::new(EpochPtr::default());
        let entity = Entity::new(
            EntityIdentifier::new("User".to_string()),
            vec![AttributeDescriptor { kind: AttributeKind::Physical, name: String::from("name"), initial: DatabaseValue::String("john".to_string()) }],
            initial_ptr,
            current_ptr,
        );
        assert!(entity.get("oops").is_err());
        assert_eq!(entity.get("oops").unwrap_err(), EntityError::AttributeNotFound("oops".to_string()))
    }
}