import tomllib
from collections.abc import Mapping
from typing import cast

from ordered import FrozenList


def table(value: object) -> Mapping[str, object]:
    if not isinstance(value, Mapping):
        message = f"expected a table, found {type(value).__name__}"
        raise TypeError(message)
    return cast("Mapping[str, object]", value)


def array(value: object) -> FrozenList[object]:
    if not isinstance(value, list):
        message = f"expected an array, found {type(value).__name__}"
        raise TypeError(message)
    return FrozenList(cast("list[object]", value))


def text(value: object) -> str:
    if not isinstance(value, str):
        message = f"expected a string, found {type(value).__name__}"
        raise TypeError(message)
    return value


def is_table(value: object) -> bool:
    return isinstance(value, Mapping)


def field(value: object, key: str) -> object:
    return table(value)[key]


def tables(value: object, key: str) -> FrozenList[Mapping[str, object]]:
    return FrozenList(table(item) for item in array(field(value, key)))


def texts(value: object, key: str) -> FrozenList[str]:
    return FrozenList(text(item) for item in array(field(value, key)))


def toml(source: str) -> Mapping[str, object]:
    return cast("Mapping[str, object]", tomllib.loads(source))


def optional_tables(
    value: Mapping[str, object], key: str
) -> FrozenList[Mapping[str, object]]:
    return FrozenList(table(item) for item in array(value.get(key, [])))


def optional_texts(value: Mapping[str, object], key: str) -> FrozenList[str]:
    return FrozenList(text(item) for item in array(value.get(key, [])))
