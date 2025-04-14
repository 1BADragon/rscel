from typing import Any, Callable, Tuple


CelBasicType = int | float | str | bool | None
CelArrayType = list[CelBasicType | 'CelArrayType' | 'CelDict']
CelDict = dict[str, 'CelValue']
CelValue = CelDict | CelArrayType | CelBasicType

CelCallable = Callable[[*Tuple[CelValue, ...]], CelValue]

CelBinding = dict[str, CelValue | CelCallable | Any]

def eval(prog: str, binding: CelBinding) -> CelValue:
    ...

class CelProgram:
    def __init__(self):
        ...

    def add_source(self, source: str):
        ...

    def add_serialized_json(self, source: str):
        ...

    def add_serialized_bincode(self, bincode: bytes):
        ...

    def serialize_to_json(self) -> str:
        ...

    def serialize_to_bincode(self) -> bytes:
        ...

    def details_json(self) -> str:
        ...

class BindContext:
    def __init__(self):
        ...

    def bind_param(self, name: str, val: CelValue):
        ...

    def bind_func(self, name: str, val: Callable[[Any, Any]]):
        ...

    def bind(self, name: str, val: Any):
        ...

class CelContext:
    def __init__(self):
        ...

    def add_program_string(self, name: str, source: str):
        ...

    def add_program(self, name: str, prog: "CelProgram"):
        ...

    def exec(self, name: str, bindings: BindContext) -> CelValue:
        ...
