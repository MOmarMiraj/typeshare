from __future__ import annotations

from enum import Enum
from pydantic import BaseModel
from typing import Literal, Union


class AutofilledByUsInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
    """
    uuid: str
    """
    The UUID for the fill
    """

class AutofilledBySomethingElseInner(BaseModel):
    """
    Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
    """
    uuid: str
    """
    The UUID for the fill
    """

class AutofilledByTypes(str, Enum):
    US = "Us"
    SOMETHING_ELSE = "SomethingElse"

class AutofilledByUs(BaseModel):
    """
    This field was autofilled by us
    """
    type: Literal[AutofilledByTypes.US] = AutofilledByTypes.US
    content: AutofilledByUsInner

class AutofilledBySomethingElse(BaseModel):
    """
    Something else autofilled this field
    """
    type: Literal[AutofilledByTypes.SOMETHING_ELSE] = AutofilledByTypes.SOMETHING_ELSE
    content: AutofilledBySomethingElseInner

# Enum keeping track of who autofilled a field
AutofilledBy = Union[AutofilledByUs, AutofilledBySomethingElse]
