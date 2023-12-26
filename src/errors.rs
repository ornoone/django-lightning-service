use crate::entity::EntityIdentifier;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EntityError {
    AttributeNotFound(String),
    EntityNotFound(EntityIdentifier),
    UnpersistedEntity(EntityIdentifier),
}
