import unittest
from nose.tools import *
from dirac import ResponeCheckError
from mysql import Module


class UnitTests(unittest.TestCase):
    def test_check_args(self):
        res = Module.check_args(3306)
        self.assertTrue(res)

    def test_none(self):
        m = Module(3306)
        res = m.check_response(None)
        self.assertFalse(res)

    def test_announced_zero_payload(self):
        m = Module(3306)
        res = m.check_response(bytearray.fromhex('00 00 00 00'))
        self.assertFalse(res)

    def test_payload_length_mismatch(self):
        m = Module(3306)
        res = m.check_response(bytearray.fromhex('01 00 00 00 aa bb'))
        self.assertFalse(res)

    def test_payload_length_2(self):
        m = Module(3306)
        res = m.check_response(bytearray.fromhex('02 00 00 00 aa bb'))
        self.assertFalse(res)

    def test_payload_length_256(self):
        m = Module(3306)
        payload = ""
        for i in range(256):
            payload += "aa "
        res = m.check_response(bytearray.fromhex('00 01 00 00 ' + payload))
        self.assertFalse(res)

    def test_valid_error_packet(self):
        m = Module(3306)
        res = m.check_response(bytearray.fromhex('09 00 00 00   FF 01 00 AA BB BB BB BB BB'))
        self.assertTrue(res)

    def test_too_short_error_packet(self):
        m = Module(3306)
        res = m.check_response(bytearray.fromhex('08 00 00 00   FF 01 00 AA BB BB BB BB'))
        self.assertFalse(res)

    def test_initial_handshake_packet(self):
        hex = "14 00 00 00"
        hex += "0a   41 42 43 44 45 00   01 00 00 00   46 47 48 49 4A 4B 4C 4E   00"
        m = Module(3306)
        res = m.check_response(bytearray.fromhex(hex))
        self.assertTrue(res)

    @raises(ResponeCheckError)
    def test_value_error(self):
        m = Module(3306)
        res = m.check_response("not a byte array")


    def test_init(self):
        m = Module(3306)
        res = m.check_response(None)
        self.assertFalse(res)

class ExampleTests(unittest.TestCase):

    def test_mysql_55_error_response(self):
        m = Module(33306)

        # Host is not allow to connect error response
        hex  = "41 00 00 00 ff 6a 04 48  6f 73 74 20 27 31 30 2e" #  |A....j.Host '10.|
        hex += "30 2e 32 2e 32 27 20 69  73 20 6e 6f 74 20 61 6c" #  |0.2.2' is not al|
        hex += "6c 6f 77 65 64 20 74 6f  20 63 6f 6e 6e 65 63 74" #  |lowed to connect|
        hex += "20 74 6f 20 74 68 69 73  20 4d 79 53 51 4c 20 73" #  | to this MySQL s|
        hex += "65 72 76 65 72                                  " #  |erver|
        # 0x45 == 69 bytes, 0x41 = 65 payload

        res = m.check_response(bytearray.fromhex(hex))
        self.assertTrue(res)

    def test_mysql_55_response(self):
        m = Module(33306)

        hex  = "5b 00 00 00 0a 35 2e 35  2e 34 39 2d 30 75 62 75" #  |[....5.5.49-0ubu|
        hex += "6e 74 75 30 2e 31 34 2e  30 34 2e 31 00 4b 00 00" #  |ntu0.14.04.1.K..|
        hex += "00 52 6c 49 67 4b 22 66  5e 00 ff f7 08 02 00 0f" #  |.RlIgK"f^.......|
        hex += "80 15 00 00 00 00 00 00  00 00 00 00 25 39 31 2a" #  |............%91*|
        hex += "7c 54 70 55 38 57 31 2b  00 6d 79 73 71 6c 5f 6e" #  ||TpU8W1+.mysql_n|
        hex += "61 74 69 76 65 5f 70 61  73 73 77 6f 72 64 00   " #  |ative_password.|
        # 0x5f == 95 bytes, 0x5b = 91 payload

        res = m.check_response(bytearray.fromhex(hex))
        self.assertTrue(res)

