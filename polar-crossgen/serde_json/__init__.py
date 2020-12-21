import dataclasses
import collections
import typing
from typing import get_type_hints
import json

import serde_types as st


class TypedJsonDeserializer:
    input: any
    primitive_types = [
        st.bool,
        st.uint8,
        st.uint16,
        st.uint32,
        st.uint64,
        st.uint128,
        st.int8,
        st.int16,
        st.int32,
        st.int64,
        st.int128,
        st.float32,
        st.float64,
        st.unit,
        st.char,
        str,
        bytes,
    ]

    def __init__(self, input_val: any):
        self.input = input_val

    # noqa
    def deserialize_any(self, obj_type) -> typing.Any:
        if obj_type in self.primitive_types:
            return obj_type(self.input)
        elif hasattr(obj_type, "__origin__"):  # Generic type
            types = getattr(obj_type, "__args__")
            if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
                assert len(types) == 1
                item_type = types[0]
                assert isinstance(self.input, list)
                result = []
                for item in self.input:
                    result.append(
                        TypedJsonDeserializer(item).deserialize_any(item_type)
                    )

                return result

            elif getattr(obj_type, "__origin__") == tuple:  # Tuple
                result = []
                assert isinstance(self.input, list)
                assert len(types) == len(self.input)
                for (item_type, item) in zip(types, self.input):
                    item = TypedJsonDeserializer(item).deserialize_any(item_type)
                    result.append(item)
                return tuple(result)

            elif getattr(obj_type, "__origin__") == typing.Union:  # Option
                assert len(types) == 2 and isinstance(None, types[1])
                if self.input is None:
                    return None
                else:
                    return self.deserialize_any(types[0])

            elif getattr(obj_type, "__origin__") == dict:  # Map
                assert len(types) == 2
                result = dict()
                assert isinstance(self.input, dict)
                for k, v in self.input.items():
                    result[k] = TypedJsonDeserializer(v).deserialize_any(types[1])

                return result

            else:
                raise st.DeserializationError("Unexpected type", obj_type)

        else:
            # handle enums + structs

            if dataclasses.is_dataclass(obj_type):
                fields = dataclasses.fields(obj_type)
                types = get_type_hints(obj_type)

                # if it has a single value field
                # treat it like a newtype variant
                if len(fields) == 1 and fields[0].name == "value":
                    return obj_type(TypedJsonDeserializer(self.input).deserialize_any(types[fields[0].name]))
                    
                # regular struct or a struct enum variant
                kwargs = {
                    field.name: TypedJsonDeserializer(
                        self.input.get(field.name, None)
                    ).deserialize_any(types[field.name])
                    for field in fields
                }
                return obj_type(**kwargs)

            else:
                # type is an enum, look through subclasses to find which variant
                # to deserialize it as
                variant_name, variant_value = next(iter(self.input.items()))
                for subclass in obj_type.__subclasses__():
                    if subclass.__name__.endswith(f"__{variant_name}"):
                        return TypedJsonDeserializer(variant_value).deserialize_any(
                            subclass
                        )

                raise st.DeserializationError("Unexpected variant name", variant_name)


def deserialize_json(input_str, obj):
    deserializer = TypedJsonDeserializer(json.loads(input_str))
    return deserializer.deserialize_any(obj)
