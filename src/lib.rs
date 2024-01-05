mod entity;
mod entity_store;
mod errors;
mod expression;

use std::rc::Rc;
use std::sync::Arc;
use pyo3::AsPyPointer;
use pyo3::exceptions::{PyException};
use pyo3::prelude::*;
use crate::entity::{BaseEntityAttribute, DatabaseValue, Entity, EntityIdentifier, Epoch, Model, PhysicalAttribute, PK};
use crate::entity_store::EntityStore;
use crate::errors::EntityError;

pyo3::create_exception!(django_lightning_service, EntityNotFound, PyException);

#[derive(FromPyObject)]
enum PyDatabaseValue {
    #[pyo3(transparent)]
    String(String),
    #[pyo3(transparent)]
    Number(i64),
}


fn to_python_error(entity_error: EntityError) -> PyErr {
    Python::with_gil(|py| {
        match entity_error {
            EntityError::EntityNotFound(identifier) => PyException::new_err("oops"),
            _ => PyException::new_err("oops")
        }
    })
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
            _ => DatabaseValue::None
            // PyDatabaseValue::None => DatabaseValue::None,
        }
    }
}

impl IntoPy<Py<PyAny>> for PyDatabaseValue {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        py.None()
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
    entity_store: Arc<EntityStore>,
}

#[pymethods]
impl PyEntityStore {
    #[new]
    fn new() -> Self {
        PyEntityStore {
            entity_store: Arc::new(EntityStore::new())
        }
    }

    fn get(&self, identifier: &PyEntityIdentifier) -> Result<PyEntity, EntityError> {
        self.entity_store.get(&identifier.entity_identifier).and_then(|entity| Ok(PyEntity {entity}))
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
}


#[pyclass(unsendable)]
struct PyAttribute {
    attribute: Rc<PhysicalAttribute>,
}


#[pymethods]
impl PyAttribute {
    fn get_initial(&self) -> PyDatabaseValue {
        self.attribute.get_initial().into()
    }

    fn get_value(&self) -> PyDatabaseValue {
        self.attribute.get_value().into()
    }

    fn set_value(&self, value: PyDatabaseValue, epoch: Epoch) {
        self.attribute.set_value(value.into(), epoch);
    }
}




/// A Python module implemented in Rust.
#[pymodule]
fn django_lightning_service(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEntityStore>()?;
    m.add_class::<PyAttribute>()?;
    m.add_class::<PyEntity>()?;
    m.add("CustomError", py.get_type::<EntityNotFound>())?;
    Ok(())
}
