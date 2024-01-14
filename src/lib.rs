mod entity;
mod entity_store;
mod errors;
mod expression;

use std::cell::RefCell;
use std::rc::Rc;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyException, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyLong, PyString};
use crate::entity::{AttributeDescriptor, AttributeKind, BaseEntityAttribute, DatabaseValue, Entity, EntityIdentifier, Epoch, Model, PhysicalAttribute, PK};
use crate::entity_store::EntityStore;
use crate::errors::EntityError;
use crate::expression::{ExactExpression, FilterExpression};

pyo3::create_exception!(django_lightning_service, EntityNotFound, PyException);

#[derive(Debug)]
enum PyDatabaseValue {
    String(String),
    Number(i64),

}

impl<'source> FromPyObject<'source> for PyDatabaseValue {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if let Ok(str) = ob.downcast::<PyString>() {
            Ok(PyDatabaseValue::String(str.extract()?))
        } else if let Ok(int) = ob.downcast::<PyLong>() {
            Ok(PyDatabaseValue::Number(int.extract()?))
        } else {
            Err(PyValueError::new_err("cannot handle this type"))
        }
    }
}


impl IntoPy<PyObject> for PyDatabaseValue {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            PyDatabaseValue::String(val) => val.into_py(py),
            PyDatabaseValue::Number(val) => val.into_py(py)
        }
    }
}


fn to_python_error(entity_error: EntityError) -> PyErr {

    match entity_error {
        EntityError::EntityNotFound(identifier) => PyException::new_err(format!("EntityNotFound({})", identifier)),
        _ => PyException::new_err("oops")
    }
}

impl From<EntityError> for pyo3::PyErr {
    fn from(value: EntityError) -> Self {
        to_python_error(value)
    }
}

impl From<DatabaseValue> for PyDatabaseValue {
    fn from(value: DatabaseValue) -> Self {
        match value {
            DatabaseValue::String(str) => PyDatabaseValue::String(str),
            DatabaseValue::Number(num) => PyDatabaseValue::Number(num),
            DatabaseValue::None => PyDatabaseValue::String("".to_string()),
        }
    }
}

impl Into<DatabaseValue> for PyDatabaseValue {
    fn into(self) -> DatabaseValue {
        match self {
            PyDatabaseValue::String(str) => DatabaseValue::String(str),
            PyDatabaseValue::Number(num) => DatabaseValue::Number(num),
            // PyDatabaseValue::None => DatabaseValue::None,
        }
    }
}


#[pyclass(unsendable)]
struct PyEntityIdentifier {
    entity_identifier: EntityIdentifier,
}


#[pymethods]
impl PyEntityIdentifier {
    #[new]
    fn new(model: String, pk: Option<PK>) -> Self{
        if let Some(pk_val) = pk {
            PyEntityIdentifier {
                entity_identifier: EntityIdentifier::new_persisted(model, pk_val)
            }
        } else {
            PyEntityIdentifier {
                entity_identifier: EntityIdentifier::new(model)
            }
        }
    }

    fn has_applied_pk(&self) -> bool {
        self.entity_identifier.has_applied_pk()
    }

    fn get_uuid(&self) -> String {
        self.entity_identifier.get_uuid().to_string()
    }
    fn get_model(&self) -> &Model {
        self.entity_identifier.get_model()
    }
    fn get_applied_pk(&self) -> PK {
        *self.entity_identifier.get_applied_pk().unwrap()
    }
}

#[pyclass(unsendable)]
struct PyEntityStore {
    entity_store: Rc<RefCell<EntityStore>>,
}

#[pymethods]
impl PyEntityStore {
    #[new]
    fn new() -> Self {
        PyEntityStore {
            entity_store: Rc::new(RefCell::new(EntityStore::new()))
        }
    }

    fn get(&self, identifier: &PyEntityIdentifier) -> Result<PyEntity, EntityError> {
        self.entity_store.borrow().get(&identifier.entity_identifier).and_then(|entity| Ok(PyEntity {entity}))
    }

    pub fn filter(&self, model: Model) -> Result<Vec<PyEntity>, PyErr> {
        let expression = ExactExpression::new("name".to_string(), DatabaseValue::String("darius".to_string()));
        let result = self.entity_store.borrow().filter(model, &FilterExpression::Exact(expression));
        match result {
            Ok(entities) => Ok(entities.iter().map(|entity| PyEntity { entity: Rc::clone(entity) }).collect()),
            Err(err) => Err(err.into())
        }
    }

    pub fn instantiate_entity(&mut self, identifier: &PyEntityIdentifier) -> PyEntity {
        let attributes_descriptors: Vec<AttributeDescriptor> = vec!["name", "age"].iter().map(
            |attr| AttributeDescriptor::new(AttributeKind::Physical, attr.to_string(), DatabaseValue::String(format!("default {}", attr)))
        ).collect();
        let entity = self.entity_store.borrow_mut().instantiate_entity(identifier.entity_identifier.clone(), attributes_descriptors);
        PyEntity {entity}

    }
}

#[pyclass(unsendable)]
struct PyEntity {
    entity: Rc<Entity>,
}

#[pymethods]
impl PyEntity {
    fn get(&self, attr: &str) -> Result<PyAttribute, EntityError> {

        self.entity.get(attr).and_then(|attr| Ok(PyAttribute {
            attribute: attr.into()
        }))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.entity.get_identifier() == other.entity.get_identifier()),
            _ => Err(PyNotImplementedError::new_err("cannot compare it")),
        }
    }

}


#[pyclass(unsendable)]
struct PyAttribute {
    attribute: Rc<PhysicalAttribute>,
}


#[pymethods]
impl PyAttribute {
    #[getter]
    fn initial(&self) -> PyDatabaseValue {
        self.attribute.get_initial().into()
    }

    #[getter]
    fn value(&self) -> PyDatabaseValue {
        println!("got value {:?}", <DatabaseValue as Into<PyDatabaseValue>>::into(self.attribute.get_value()));
        self.attribute.get_value().into()
    }

    fn set_value(&self, value: PyDatabaseValue, epoch: Epoch) {
        self.attribute.set_value(value.into(), epoch);
    }



    fn __str__(slf: &PyCell<Self>) -> PyResult<String> {
        let attr = &slf.borrow().attribute;
        Ok(attr.to_string())
    }
    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {

        let class_name: String = slf.get_type().name()?.to_string();

        Ok(format!("<{} {}>", class_name, slf.borrow().attribute.to_string()))
    }
}

#[pyfunction]
fn create_database_value(type_: &str) -> PyResult<PyDatabaseValue> {

    if type_ == "Number" {
        Ok(PyDatabaseValue::Number(42))
    } else if type_ == "String" {
        Ok(PyDatabaseValue::String("world".to_string()))
    } else {
        Err(PyValueError::new_err(format!("unknown type: {}", type_)))
    }
}


#[pyfunction]
fn repr_database_value(value: PyDatabaseValue)  -> PyResult<String> {
    Ok(Into::<DatabaseValue>::into(value).to_string())
}


/// A Python module implemented in Rust.
#[pymodule]
fn django_lightning_service(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEntityStore>()?;
    m.add_class::<PyAttribute>()?;
    m.add_class::<PyEntity>()?;
    m.add_class::<PyEntityIdentifier>()?;
    m.add_function(wrap_pyfunction!(create_database_value, m)?).unwrap();
    m.add_function(wrap_pyfunction!(repr_database_value, m)?).unwrap();
    m.add("CustomError", py.get_type::<EntityNotFound>())?;
    Ok(())
}
