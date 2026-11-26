import unittest
from verify_target_wasm import parse


def section(tag, payload):
    assert len(payload) < 128
    return bytes([tag, len(payload)]) + payload


def module(result=0x7e, start_params=b"\0", start_section=False):
    types = b"\x02\x60\x03\x7f\x7f\x7f\x01" + bytes([result])
    types += b"\x60" + start_params + b"\x00"
    imports = b"\x01\x02p0\x06invoke\x00\x00"
    funcs = b"\x01\x01"
    exports = b"\x01\x06_start\x00\x01"
    code = b"\x01\x02\x00\x0b"
    return (b"\0asm\x01\0\0\0" + section(1, types) + section(2, imports)
            + section(3, funcs) + section(7, exports)
            + (section(8, b"\x01") if start_section else b"") + section(10, code))


class SurfaceTests(unittest.TestCase):
    def test_tiny_valid_module(self):
        self.assertEqual(parse(module()), ["_start"])

    def test_wrong_import_result(self):
        with self.assertRaisesRegex(ValueError, "p0.invoke must have type"):
            parse(module(result=0x7f))

    def test_wrong_start_params(self):
        with self.assertRaisesRegex(ValueError, "_start must be"):
            parse(module(start_params=b"\x01\x7f"))

    def test_start_section_rejected(self):
        with self.assertRaisesRegex(ValueError, "start section"):
            parse(module(start_section=True))

    def test_truncated_and_duplicate_sections(self):
        for malformed in [module()[:-1], module() + section(7, b"\0")]:
            with self.assertRaises(ValueError):
                parse(malformed)


if __name__ == "__main__":
    unittest.main()
