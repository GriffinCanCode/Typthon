"""Test collection type checking.

This module tests type checking for lists, tuples, dicts, and sets.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestListTypes:
    """Test list type checking."""

    def test_empty_list(self, validator):
        """Test empty list."""
        code = "x: list[int] = []"
        assert validator.validate(code)

    def test_list_of_ints(self, validator):
        """Test list of integers."""
        code = "x: list[int] = [1, 2, 3]"
        assert validator.validate(code)

    def test_list_of_strings(self, validator):
        """Test list of strings."""
        code = 'x: list[str] = ["a", "b", "c"]'
        assert validator.validate(code)

    def test_list_of_floats(self, validator):
        """Test list of floats."""
        code = "x: list[float] = [1.0, 2.0, 3.0]"
        assert validator.validate(code)

    def test_list_type_error(self, validator):
        """Test list type mismatch."""
        code = 'x: list[int] = ["a", "b"]'
        assert not validator.validate(code)

    def test_list_mixed_types_error(self, validator):
        """Test list with mixed types error."""
        code = 'x: list[int] = [1, "two", 3]'
        assert not validator.validate(code)

    def test_nested_list(self, validator):
        """Test nested lists."""
        code = "x: list[list[int]] = [[1, 2], [3, 4]]"
        assert validator.validate(code)

    def test_list_append(self, validator):
        """Test list append method."""
        code = """
x: list[int] = [1, 2]
x.append(3)
"""
        assert validator.validate(code)

    def test_list_append_wrong_type(self, validator):
        """Test list append with wrong type."""
        code = """
x: list[int] = [1, 2]
x.append("three")  # Type error
"""
        assert not validator.validate(code)

    def test_list_extend(self, validator):
        """Test list extend method."""
        code = """
x: list[int] = [1, 2]
x.extend([3, 4])
"""
        assert validator.validate(code)

    def test_list_pop(self, validator):
        """Test list pop method."""
        code = """
x: list[int] = [1, 2, 3]
item: int = x.pop()
"""
        assert validator.validate(code)

    def test_list_indexing(self, validator):
        """Test list indexing."""
        code = """
x: list[int] = [1, 2, 3]
item: int = x[0]
"""
        assert validator.validate(code)

    def test_list_slicing(self, validator):
        """Test list slicing."""
        code = """
x: list[int] = [1, 2, 3, 4, 5]
sublist: list[int] = x[1:3]
"""
        assert validator.validate(code)

    def test_list_comprehension(self, validator):
        """Test list comprehension."""
        code = """
x: list[int] = [i * 2 for i in range(5)]
"""
        assert validator.validate(code)

    def test_list_comprehension_with_filter(self, validator):
        """Test list comprehension with filter."""
        code = """
x: list[int] = [i for i in range(10) if i % 2 == 0]
"""
        assert validator.validate(code)

    def test_list_len(self, validator):
        """Test len() on list."""
        code = """
x: list[int] = [1, 2, 3]
length: int = len(x)
"""
        assert validator.validate(code)

    def test_list_in_operator(self, validator):
        """Test 'in' operator on list."""
        code = """
x: list[int] = [1, 2, 3]
result: bool = 2 in x
"""
        assert validator.validate(code)

    def test_list_iteration(self, validator):
        """Test iterating over list."""
        code = """
items: list[int] = [1, 2, 3]
for item in items:
    x: int = item
"""
        assert validator.validate(code)

    def test_list_sort(self, validator):
        """Test list sort method."""
        code = """
x: list[int] = [3, 1, 2]
x.sort()
"""
        assert validator.validate(code)

    def test_list_reverse(self, validator):
        """Test list reverse method."""
        code = """
x: list[int] = [1, 2, 3]
x.reverse()
"""
        assert validator.validate(code)

    def test_list_copy(self, validator):
        """Test list copy method."""
        code = """
x: list[int] = [1, 2, 3]
y: list[int] = x.copy()
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestTupleTypes:
    """Test tuple type checking."""

    def test_empty_tuple(self, validator):
        """Test empty tuple."""
        code = "x: tuple[()] = ()"
        assert validator.validate(code)

    def test_tuple_single_element(self, validator):
        """Test single element tuple."""
        code = "x: tuple[int] = (1,)"
        assert validator.validate(code)

    def test_tuple_multiple_elements(self, validator):
        """Test tuple with multiple elements."""
        code = "x: tuple[int, str, float] = (1, 'hello', 3.14)"
        assert validator.validate(code)

    def test_tuple_homogeneous(self, validator):
        """Test homogeneous tuple."""
        code = "x: tuple[int, int, int] = (1, 2, 3)"
        assert validator.validate(code)

    def test_tuple_type_error(self, validator):
        """Test tuple type mismatch."""
        code = "x: tuple[int, str] = (1, 2)"
        assert not validator.validate(code)

    def test_tuple_length_error(self, validator):
        """Test tuple length mismatch."""
        code = "x: tuple[int, int] = (1, 2, 3)"
        assert not validator.validate(code)

    def test_nested_tuple(self, validator):
        """Test nested tuples."""
        code = "x: tuple[int, tuple[str, str]] = (1, ('a', 'b'))"
        assert validator.validate(code)

    def test_tuple_indexing(self, validator):
        """Test tuple indexing."""
        code = """
x: tuple[int, str, float] = (1, 'hello', 3.14)
first: int = x[0]
second: str = x[1]
"""
        assert validator.validate(code)

    def test_tuple_unpacking(self, validator):
        """Test tuple unpacking."""
        code = """
x: tuple[int, str] = (1, 'hello')
a: int
b: str
a, b = x
"""
        assert validator.validate(code)

    def test_tuple_unpacking_error(self, validator):
        """Test tuple unpacking type error."""
        code = """
x: tuple[int, str] = (1, 'hello')
a: str
b: int
a, b = x  # Type error
"""
        assert not validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestDictTypes:
    """Test dict type checking."""

    def test_empty_dict(self, validator):
        """Test empty dict."""
        code = "x: dict[str, int] = {}"
        assert validator.validate(code)

    def test_dict_str_int(self, validator):
        """Test dict with string keys and int values."""
        code = "x: dict[str, int] = {'a': 1, 'b': 2}"
        assert validator.validate(code)

    def test_dict_int_str(self, validator):
        """Test dict with int keys and string values."""
        code = "x: dict[int, str] = {1: 'a', 2: 'b'}"
        assert validator.validate(code)

    def test_dict_key_type_error(self, validator):
        """Test dict key type mismatch."""
        code = "x: dict[str, int] = {1: 1, 2: 2}"
        assert not validator.validate(code)

    def test_dict_value_type_error(self, validator):
        """Test dict value type mismatch."""
        code = "x: dict[str, int] = {'a': 'one', 'b': 'two'}"
        assert not validator.validate(code)

    def test_dict_indexing(self, validator):
        """Test dict indexing."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
value: int = x['a']
"""
        assert validator.validate(code)

    def test_dict_get_method(self, validator):
        """Test dict get method."""
        code = """
x: dict[str, int] = {'a': 1}
value: int = x.get('a')
"""
        assert validator.validate(code)

    def test_dict_keys(self, validator):
        """Test dict keys method."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
keys = x.keys()
"""
        assert validator.validate(code)

    def test_dict_values(self, validator):
        """Test dict values method."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
values = x.values()
"""
        assert validator.validate(code)

    def test_dict_items(self, validator):
        """Test dict items method."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
items = x.items()
"""
        assert validator.validate(code)

    def test_dict_iteration(self, validator):
        """Test iterating over dict."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
for key in x:
    value: int = x[key]
"""
        assert validator.validate(code)

    def test_dict_in_operator(self, validator):
        """Test 'in' operator on dict."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
result: bool = 'a' in x
"""
        assert validator.validate(code)

    def test_dict_update(self, validator):
        """Test dict update method."""
        code = """
x: dict[str, int] = {'a': 1}
x.update({'b': 2})
"""
        assert validator.validate(code)

    def test_dict_pop(self, validator):
        """Test dict pop method."""
        code = """
x: dict[str, int] = {'a': 1, 'b': 2}
value: int = x.pop('a')
"""
        assert validator.validate(code)

    def test_dict_comprehension(self, validator):
        """Test dict comprehension."""
        code = """
x: dict[int, int] = {i: i * 2 for i in range(5)}
"""
        assert validator.validate(code)

    def test_nested_dict(self, validator):
        """Test nested dicts."""
        code = """
x: dict[str, dict[str, int]] = {
    'a': {'x': 1, 'y': 2},
    'b': {'x': 3, 'y': 4}
}
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestSetTypes:
    """Test set type checking."""

    def test_empty_set(self, validator):
        """Test empty set."""
        code = "x: set[int] = set()"
        assert validator.validate(code)

    def test_set_of_ints(self, validator):
        """Test set of integers."""
        code = "x: set[int] = {1, 2, 3}"
        assert validator.validate(code)

    def test_set_of_strings(self, validator):
        """Test set of strings."""
        code = "x: set[str] = {'a', 'b', 'c'}"
        assert validator.validate(code)

    def test_set_type_error(self, validator):
        """Test set type mismatch."""
        code = "x: set[int] = {'a', 'b'}"
        assert not validator.validate(code)

    def test_set_add(self, validator):
        """Test set add method."""
        code = """
x: set[int] = {1, 2}
x.add(3)
"""
        assert validator.validate(code)

    def test_set_remove(self, validator):
        """Test set remove method."""
        code = """
x: set[int] = {1, 2, 3}
x.remove(2)
"""
        assert validator.validate(code)

    def test_set_discard(self, validator):
        """Test set discard method."""
        code = """
x: set[int] = {1, 2, 3}
x.discard(2)
"""
        assert validator.validate(code)

    def test_set_union(self, validator):
        """Test set union."""
        code = """
x: set[int] = {1, 2}
y: set[int] = {2, 3}
z: set[int] = x.union(y)
"""
        assert validator.validate(code)

    def test_set_intersection(self, validator):
        """Test set intersection."""
        code = """
x: set[int] = {1, 2, 3}
y: set[int] = {2, 3, 4}
z: set[int] = x.intersection(y)
"""
        assert validator.validate(code)

    def test_set_difference(self, validator):
        """Test set difference."""
        code = """
x: set[int] = {1, 2, 3}
y: set[int] = {2, 3}
z: set[int] = x - y
"""
        assert validator.validate(code)

    def test_set_in_operator(self, validator):
        """Test 'in' operator on set."""
        code = """
x: set[int] = {1, 2, 3}
result: bool = 2 in x
"""
        assert validator.validate(code)

    def test_set_iteration(self, validator):
        """Test iterating over set."""
        code = """
items: set[int] = {1, 2, 3}
for item in items:
    x: int = item
"""
        assert validator.validate(code)

    def test_set_comprehension(self, validator):
        """Test set comprehension."""
        code = """
x: set[int] = {i * 2 for i in range(5)}
"""
        assert validator.validate(code)

    def test_set_len(self, validator):
        """Test len() on set."""
        code = """
x: set[int] = {1, 2, 3}
length: int = len(x)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestComplexCollections:
    """Test complex collection combinations."""

    def test_list_of_tuples(self, validator):
        """Test list of tuples."""
        code = """
x: list[tuple[int, str]] = [(1, 'a'), (2, 'b'), (3, 'c')]
"""
        assert validator.validate(code)

    def test_dict_of_lists(self, validator):
        """Test dict of lists."""
        code = """
x: dict[str, list[int]] = {
    'a': [1, 2, 3],
    'b': [4, 5, 6]
}
"""
        assert validator.validate(code)

    def test_list_of_dicts(self, validator):
        """Test list of dicts."""
        code = """
x: list[dict[str, int]] = [
    {'a': 1, 'b': 2},
    {'c': 3, 'd': 4}
]
"""
        assert validator.validate(code)

    def test_nested_collections(self, validator):
        """Test deeply nested collections."""
        code = """
x: dict[str, list[tuple[int, str]]] = {
    'group1': [(1, 'a'), (2, 'b')],
    'group2': [(3, 'c'), (4, 'd')]
}
"""
        assert validator.validate(code)

    def test_set_of_tuples(self, validator):
        """Test set of tuples."""
        code = """
x: set[tuple[int, str]] = {(1, 'a'), (2, 'b'), (3, 'c')}
"""
        assert validator.validate(code)

    def test_mixed_collection_operations(self, validator):
        """Test operations on mixed collections."""
        code = """
data: dict[str, list[int]] = {'nums': [1, 2, 3]}
nums: list[int] = data['nums']
first: int = nums[0]
"""
        assert validator.validate(code)

