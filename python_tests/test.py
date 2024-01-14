
import pytest

from django_lightning_service import PyEntityIdentifier, PyEntityStore, create_database_value, repr_database_value


@pytest.fixture()
def entity_store():
    return PyEntityStore()


def test_instantiate_then_get(entity_store):
    ident = PyEntityIdentifier("Model")
    entity = entity_store.instantiate_entity(ident)

    entity.get('name').set_value('darius', 1)
    print(entity.get("name"))

    assert entity.get('name').value == 'darius'

    other_entity = entity_store.get(ident)
    assert entity == other_entity



def test_database_value_into_python():
    assert create_database_value("String") == "world"
    assert create_database_value("Number") == 42


def test_python_todatabase_value():
    assert repr_database_value("world") == "String(world)"
    assert repr_database_value(42) == "Number(42)"