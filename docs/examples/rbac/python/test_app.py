from pathlib import Path


def test_policy(
    test_oso, ariana, bhavik, affine_types, allocator, bubble_sort, benchmarks
):
    test_oso.load_files([Path(__file__).parent / "main.polar"])

    assert test_oso.is_allowed(ariana, "read", affine_types)
    assert test_oso.is_allowed(ariana, "push", affine_types)
    assert test_oso.is_allowed(ariana, "read", allocator)
    assert test_oso.is_allowed(ariana, "push", allocator)
    assert not test_oso.is_allowed(ariana, "read", bubble_sort)
    assert not test_oso.is_allowed(ariana, "push", bubble_sort)
    assert not test_oso.is_allowed(ariana, "read", benchmarks)
    assert not test_oso.is_allowed(ariana, "push", benchmarks)

    assert not test_oso.is_allowed(bhavik, "read", affine_types)
    assert not test_oso.is_allowed(bhavik, "push", affine_types)
    assert not test_oso.is_allowed(bhavik, "read", allocator)
    assert not test_oso.is_allowed(bhavik, "push", allocator)
    assert test_oso.is_allowed(bhavik, "read", bubble_sort)
    assert not test_oso.is_allowed(bhavik, "push", bubble_sort)
    assert test_oso.is_allowed(bhavik, "read", benchmarks)
    assert test_oso.is_allowed(bhavik, "push", benchmarks)
