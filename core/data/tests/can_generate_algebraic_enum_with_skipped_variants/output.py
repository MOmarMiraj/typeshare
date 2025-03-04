from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import Literal, Union


class SomeEnumTypes(str, Enum):
    A = "A"
    C = "C"

class SomeEnumA(BaseModel):
    type: Literal[SomeEnumTypes.A] = SomeEnumTypes.A

class SomeEnumC(BaseModel):
    type: Literal[SomeEnumTypes.C] = SomeEnumTypes.C
    content: int

SomeEnum = Union[SomeEnumA, SomeEnumC]
