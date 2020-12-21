import unittest
import serde_types as st


class SerdeTypesTestCase(unittest.TestCase):
    def test_u128(self):
        x = 0x0102030405060708090A0B0C0D0E0F10
        y = st.uint128(x)
        self.assertEqual(y.high, 0x0102030405060708)
        self.assertEqual(y.low, 0x090A0B0C0D0E0F10)
        self.assertEqual(int(y), x)

    def test_i128_positive(self):
        x = 0x0102030405060708090A0B0C0D0E0F10
        y = st.int128(x)
        self.assertEqual(y.high, 0x0102030405060708)
        self.assertEqual(y.low, 0x090A0B0C0D0E0F10)
        self.assertEqual(int(y), x)

    def test_i128_negative(self):
        x = -2
        y = st.int128(x)
        self.assertEqual(y.high, -1)
        self.assertEqual(y.low, 0xFFFFFFFFFFFFFFFE)
        self.assertEqual(int(y), x)

    def test_char(self):
        self.assertEqual(str(st.char("a")), "a")
        with self.assertRaises(ValueError):
            st.char("ab")
