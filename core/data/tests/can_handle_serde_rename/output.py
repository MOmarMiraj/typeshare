from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field
from typing import List, Optional


class OtherType(BaseModel):
    pass
class Person(BaseModel):
    """
    This is a comment.
    """
    model_config = ConfigDict(populate_by_name=True)

    name: str
    age: int
    extra_special_field_1: int = Field(alias="extraSpecialFieldOne")
    extra_special_field_2: Optional[List[str]] = Field(alias="extraSpecialFieldTwo", default=None)
    non_standard_data_type: OtherType = Field(alias="nonStandardDataType")
    non_standard_data_type_in_array: Optional[List[OtherType]] = Field(alias="nonStandardDataTypeInArray", default=None)

