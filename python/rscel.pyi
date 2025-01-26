from typing import Any, Callable, Tuple


CelBasicType = int | float | str | bool | None
CelArrayType = list[CelBasicType | 'CelArrayType' | 'CelDict']
CelDict = dict[str, 'CelValue']
CelValue = CelDict | CelArrayType | CelBasicType

CelCallable = Callable[[*Tuple[CelValue, ...]], CelValue]

CelBinding = dict[str, CelValue | CelCallable | Any]

def eval(prog: str, binding: CelBinding) -> CelValue:
    ...
