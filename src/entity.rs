use std::cell::RefCell;
use std::collections::HashMap;

type Epoch = i64;

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


struct PhysicalAttribute<'a, T> {
    current_epoch_ptr: &'a EpochPtr,

    initial_epoch_ptr: &'a EpochPtr,

    value_history: Vec<AttributeValue<T>>,
}

impl<'a, T> PhysicalAttribute<'a, T> {
    fn new(current_epoch_ptr: &'a EpochPtr, initial_epoch_ptr: &'a EpochPtr) -> Self {
        PhysicalAttribute {
            current_epoch_ptr,
            initial_epoch_ptr,
            value_history: vec!(),
        }
    }


    fn get_at_epoch(&self, epoch: Epoch) -> &T {

        for history in self.value_history.iter().rev() {
            if history.epoch <= epoch {
                return &history.value;
            }
        }
        // return the initial value instead
        let initial = self.value_history.first().unwrap();
        return &initial.value;

    }

    fn insert_at_epoch(&mut self, value: T, epoch: Epoch) {
        let history_value = AttributeValue{epoch, value};
        for (i, hist) in self.value_history.iter().enumerate() {
            if hist.epoch > epoch {
                self.value_history.insert(i, history_value);
                return;
            }
        }

        self.value_history.push(history_value);
    }

    fn get_initial(&self) -> &T {
        self.get_at_epoch(self.initial_epoch_ptr.get_epoch())
    }

    fn get_value(&self) -> &T {
        self.get_at_epoch(self.current_epoch_ptr.get_epoch())
    }

    fn set_value(&mut self, value: T, epoch: Epoch) {
        self.insert_at_epoch(value, epoch);
    }
}


struct Entity<'a> {
    physical_attributes: HashMap<&'a str, PhysicalAttribute<'a, DatabaseValue>>,
}

#[cfg(test)]
mod tests {
    use crate::entity::{DatabaseValue, EpochPtr, PhysicalAttribute};

    #[test]
    fn get_ptr_slide() {
        let initial_ptr = EpochPtr::default();
        let current_ptr = EpochPtr::default();
        current_ptr.slide(2);
        let mut attr: PhysicalAttribute<DatabaseValue> = PhysicalAttribute::new(&current_ptr, &initial_ptr);
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
}