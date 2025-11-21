"""Integration tests for Python-Rust bridge."""

import pytest
import sys
from pathlib import Path


class TestRustExtensionIntegration:
    """Test Python ↔ Rust integration through PyO3."""

    def test_import_rust_extension(self):
        """Test that Rust extension can be imported."""
        try:
            from typthon.typthon import TypeValidator
            assert TypeValidator is not None
        except ImportError as e:
            pytest.skip(f"Rust extension not available: {e}")

    def test_type_validator_basic(self):
        """Test basic TypeValidator functionality."""
        from typthon.typthon import TypeValidator

        validator = TypeValidator()
        assert validator is not None

        # Valid code should validate
        result = validator.validate("x: int = 5")
        assert isinstance(result, bool)

    def test_type_validator_get_type(self):
        """Test TypeValidator.get_type()."""
        from typthon.typthon import TypeValidator

        validator = TypeValidator()
        validator.validate("x: int = 5")

        # Get type should return something
        result = validator.get_type("5")
        assert isinstance(result, str)

    def test_check_file_py(self):
        """Test check_file_py function."""
        from typthon.typthon import check_file_py

        # Create temporary test file
        test_file = Path(__file__).parent / "temp_test.py"
        test_file.write_text("x: int = 5\ny: str = 'hello'")

        try:
            errors = check_file_py(str(test_file))
            assert isinstance(errors, list)
        finally:
            test_file.unlink(missing_ok=True)

    def test_infer_types_py(self):
        """Test infer_types_py function."""
        from typthon.typthon import infer_types_py

        result = infer_types_py("x = 5")
        assert isinstance(result, str)

    def test_analyze_effects_py(self):
        """Test analyze_effects_py function."""
        from typthon.typthon import analyze_effects_py

        result = analyze_effects_py("def f(x): return x + 1")
        assert isinstance(result, dict)

    def test_validate_refinement_py(self):
        """Test validate_refinement_py function."""
        from typthon.typthon import validate_refinement_py

        result = validate_refinement_py("5", "x > 0")
        assert isinstance(result, bool)


class TestRuntimeFunctions:
    """Test runtime management functions."""

    def test_init_runtime(self):
        """Test runtime initialization."""
        from typthon.typthon import init_runtime_py

        # Should not raise
        init_runtime_py()
        init_runtime_py()  # Second call should be safe

    def test_get_runtime_stats(self):
        """Test getting runtime statistics."""
        from typthon.typthon import get_runtime_stats, init_runtime_py

        init_runtime_py()
        stats = get_runtime_stats()

        assert hasattr(stats, "gc_collections")
        assert hasattr(stats, "heap_allocated")
        assert hasattr(stats, "cache_hits")
        assert hasattr(stats, "cache_misses")
        assert hasattr(stats, "uptime_secs")

        assert isinstance(stats.gc_collections, int)
        assert isinstance(stats.heap_allocated, int)
        assert stats.uptime_secs >= 0

    def test_force_gc(self):
        """Test GC forcing."""
        from typthon.typthon import force_gc_py, get_runtime_stats, init_runtime_py

        init_runtime_py()
        initial_stats = get_runtime_stats()
        initial_gc = initial_stats.gc_collections

        force_gc_py()

        new_stats = get_runtime_stats()
        assert new_stats.gc_collections >= initial_gc

    def test_clear_cache(self):
        """Test cache clearing."""
        from typthon.typthon import clear_cache_py

        result = clear_cache_py()
        assert isinstance(result, str)
        assert "cleared" in result.lower()

    def test_get_metrics(self):
        """Test metrics retrieval."""
        from typthon.typthon import get_metrics_py, init_runtime_py

        init_runtime_py()
        metrics = get_metrics_py()

        assert isinstance(metrics, dict)
        assert "uptime" in metrics


class TestTopLevelAPI:
    """Test top-level package API."""

    def test_type_validator_accessible(self):
        """Test that TypeValidator is accessible from top level."""
        import typthon

        assert hasattr(typthon, "TypeValidator")
        if typthon.TypeValidator is not None:
            validator = typthon.TypeValidator()
            assert validator is not None

    def test_rust_functions_accessible(self):
        """Test that Rust functions are accessible from top level."""
        import typthon

        assert hasattr(typthon, "check_file_py")
        assert hasattr(typthon, "infer_types_py")
        assert hasattr(typthon, "analyze_effects_py")
        assert hasattr(typthon, "init_runtime_py")
        assert hasattr(typthon, "get_runtime_stats")
        assert hasattr(typthon, "force_gc_py")
        assert hasattr(typthon, "clear_cache_py")
        assert hasattr(typthon, "get_metrics_py")


class TestTypeConstructs:
    """Test type construct subscriptability."""

    def test_union_subscriptable(self):
        """Test Union[int, str] works."""
        from typthon import Union

        # Should not raise
        union_type = Union[int, str]
        assert union_type is not None
        assert hasattr(union_type, "types")
        assert int in union_type.types
        assert str in union_type.types

    def test_intersection_subscriptable(self):
        """Test Intersection[int, str] works."""
        from typthon import Intersection

        # Should not raise
        intersection_type = Intersection[int, str]
        assert intersection_type is not None
        assert hasattr(intersection_type, "types")

    def test_optional_subscriptable(self):
        """Test Optional[int] works."""
        from typthon import Optional

        # Should not raise
        optional_type = Optional[int]
        assert optional_type is not None
        assert hasattr(optional_type, "inner")
        assert optional_type.inner == int

    def test_literal_subscriptable(self):
        """Test Literal[1, 2, 3] works."""
        from typthon import Literal

        # Should not raise
        literal_type = Literal[1, 2, 3]
        assert literal_type is not None
        assert hasattr(literal_type, "values")
        assert 1 in literal_type.values
        assert 2 in literal_type.values
        assert 3 in literal_type.values

    def test_union_single_param(self):
        """Test Union with single parameter."""
        from typthon import Union

        union_type = Union[int]
        assert union_type is not None
        assert int in union_type.types


class TestEndToEnd:
    """End-to-end integration tests."""

    def test_complete_workflow(self):
        """Test complete type checking workflow."""
        import typthon

        if typthon.TypeValidator is None:
            pytest.skip("Rust extension not available")

        # Initialize runtime
        typthon.init_runtime_py()

        # Create validator
        validator = typthon.TypeValidator()

        # Validate some code
        code = """
def add(x: int, y: int) -> int:
    return x + y

result: int = add(1, 2)
"""
        is_valid = validator.validate(code)
        assert isinstance(is_valid, bool)

        # Get type
        type_result = validator.get_type("1 + 2")
        assert isinstance(type_result, str)

        # Get stats
        stats = typthon.get_runtime_stats()
        assert stats.uptime_secs >= 0

    def test_error_handling(self):
        """Test error handling in validation."""
        import typthon

        if typthon.TypeValidator is None:
            pytest.skip("Rust extension not available")

        validator = typthon.TypeValidator()

        # Invalid syntax should handle gracefully
        with pytest.raises(Exception):
            validator.validate("def broken(")

    def test_multiple_validators(self):
        """Test multiple validators can coexist."""
        import typthon

        if typthon.TypeValidator is None:
            pytest.skip("Rust extension not available")

        validator1 = typthon.TypeValidator()
        validator2 = typthon.TypeValidator()

        code = "x: int = 5"
        result1 = validator1.validate(code)
        result2 = validator2.validate(code)

        assert result1 == result2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

