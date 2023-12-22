use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;

type Epoch = i64;

#[derive(Debug)]

struct EpochPtr {
    epoch: RefCell<Epoch>,
}

impl Default for EpochPtr {
    fn default() -> Self {
        EpochPtr { epoch: RefCell::new(0) }
    }
}

impl EpochPtr {
    fn slide(&self, epoch: Epoch) {
        let mut ep = self.epoch.borrow_mut();
        *ep = epoch;
    }

    fn get_epoch(&self) -> Epoch {
        *self.epoch.borrow()
    }
}

#[derive(Debug)]

struct AttributeValue<T> {
    epoch: Epoch,
    value: T,
}

#[derive(PartialEq)]
#[derive(Debug)]
enum DatabaseValue {
    String(String),
    Number(i64),
}

trait BaseEntityAttribute {
    fn get_initial(&self) -> &DatabaseValue;

    fn get_value(&self) -> &DatabaseValue;

    fn set_value(&mut self, value: DatabaseValue, epoch: Epoch);
}

pub trait EntityAttribute: Debug + BaseEntityAttribute {}

impl<T: Debug + BaseEntityAttribute> EntityAttribute for T {}

#[derive(Debug)]
struct PhysicalAttribute<'a> {
    current_epoch_ptr: &'a EpochPtr,

    initial_epoch_ptr: &'a EpochPtr,

    value_history: Vec<AttributeValue<DatabaseValue>>,
}

impl<'a> PhysicalAttribute<'a> {
    fn new(current_epoch_ptr: &'a EpochPtr, initial_epoch_ptr: &'a EpochPtr) -> Self {
        PhysicalAttribute {
            current_epoch_ptr,
            initial_epoch_ptr,
            value_history: vec!(),
        }
    }


    fn get_at_epoch(&self, epoch: Epoch) -> &DatabaseValue {
        for history in self.value_history.iter().rev() {
            if history.epoch <= epoch {
                return &history.value;
            }
        }
        // return the initial value instead
        let initial = self.value_history.first().unwrap();
        return &initial.value;
    }

    fn insert_at_epoch(&mut self, value: DatabaseValue, epoch: Epoch) {
        let history_value = AttributeValue { epoch, value };
        for (i, hist) in self.value_history.iter().enumerate() {
            if hist.epoch > epoch {
                self.value_history.insert(i, history_value);
                return;
            }
        }

        self.value_history.push(history_value);
    }
}


impl<'a> BaseEntityAttribute for PhysicalAttribute<'a> {
    fn get_initial(&self) -> &DatabaseValue {
        self.get_at_epoch(self.initial_epoch_ptr.get_epoch())
    }

    fn get_value(&self) -> &DatabaseValue {
        self.get_at_epoch(self.current_epoch_ptr.get_epoch())
    }

    fn set_value(&mut self, value: DatabaseValue, epoch: Epoch) {
        self.insert_at_epoch(value, epoch);
    }
}


struct Entity<'a> {
    physical_attributes: HashMap<String, PhysicalAttribute<'a>>,

}

#[derive(Debug)]
#[derive(PartialEq)]
enum EntityError {
    AttributeNotFound(String),
}

enum AttributeKind {
    Physical,
    ManyToMany,
}

struct AttributeDescriptor {
    kind: AttributeKind,
    name: String,
    initial: DatabaseValue,
}

impl<'a> Entity<'a> {
    fn new(attributes: Vec<AttributeDescriptor>, initial_ptr: &'a EpochPtr, current_ptr: &'a EpochPtr) -> Self {
        let mut physicals: HashMap<String, PhysicalAttribute> = HashMap::new();

        for attribute in attributes {
            match attribute.kind {
                AttributeKind::ManyToMany => panic!("not yet implemented"),
                AttributeKind::Physical => {
                    let mut attr = PhysicalAttribute::new(current_ptr, initial_ptr);
                    attr.set_value(attribute.initial, initial_ptr.get_epoch());
                    physicals.insert(attribute.name, attr);
                }
            }
        }
        Entity {
            physical_attributes: physicals
        }
    }

    fn get<'b>(&'a self, attribute: &'b str) -> Result<&'a (dyn EntityAttribute), EntityError> {
        if let Some(attr) = self.physical_attributes.get(attribute) {
            Ok(attr)
        } else {
            Err(EntityError::AttributeNotFound(attribute.to_string()))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::entity::{DatabaseValue, EpochPtr, PhysicalAttribute, BaseEntityAttribute, Entity, AttributeDescriptor, AttributeKind, EntityError};

    #[test]
    fn get_ptr_slide() {
        let initial_ptr = EpochPtr::default();
        let current_ptr = EpochPtr::default();
        current_ptr.slide(2);
        let mut attr: PhysicalAttribute = PhysicalAttribute::new(&current_ptr, &initial_ptr);
        attr.set_value(DatabaseValue::Number(42), 0);
        attr.set_value(DatabaseValue::Number(52), 2);

        assert_eq!(attr.get_initial(), &DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), &DatabaseValue::Number(52));
        current_ptr.slide(3);
        assert_eq!(attr.get_initial(), &DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), &DatabaseValue::Number(52));

        current_ptr.slide(0);
        assert_eq!(attr.get_initial(), &DatabaseValue::Number(42));
        assert_eq!(attr.get_value(), &DatabaseValue::Number(42));
    }

    #[test]
    fn test_entity() {
        let initial_ptr = EpochPtr::default();
        let current_ptr = EpochPtr::default();
        let entity = Entity::new(
            vec![AttributeDescriptor { kind: AttributeKind::Physical, name: String::from("name"), initial: DatabaseValue::String("john".to_string()) }],
            &initial_ptr,
            &current_ptr
        );

        assert_eq!(entity.get("name").unwrap().get_initial(), &DatabaseValue::String("john".to_string()))
    }


    #[test]
    fn test_attr_not_found() {
        let initial_ptr = EpochPtr::default();
        let current_ptr = EpochPtr::default();
        let entity = Entity::new(
            vec![AttributeDescriptor { kind: AttributeKind::Physical, name: String::from("name"), initial: DatabaseValue::String("john".to_string()) }],
            &initial_ptr,
            &current_ptr
        );
        assert!(entity.get("oops").is_err());
        assert_eq!(entity.get("oops").unwrap_err(), EntityError::AttributeNotFound("oops".to_string()))
    }
}