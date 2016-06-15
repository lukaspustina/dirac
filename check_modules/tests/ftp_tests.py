import unittest

from dirac.ftp import Module


class UnitTests(unittest.TestCase):
    def test_check_args(self):
        res = Module.check_args(21, 220)
        self.assertTrue(res)

    # noinspection PyMethodMayBeStatic
    def test_init(self):
        Module(21, 220)

    def test_check_response(self):
        m = Module(21, 220)
        res = m.check_response("220 FTP Server")
        self.assertTrue(res)


class ExampleTests(unittest.TestCase):
    # def test__software__version__os__version

    def test__proftpd__1_3_5__ubuntu__14_04(self):
        m = Module(21, 220)
        res = m.check_response("220 FTP Server")
        self.assertTrue(res)
