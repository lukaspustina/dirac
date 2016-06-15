import unittest

from dirac.smtp import Module


class UnitTests(unittest.TestCase):
    def test_check_args(self):
        res = Module.check_args(25, ".*Postfix.*", False, 220)
        self.assertTrue(res)

    # noinspection PyMethodMayBeStatic
    def test_init(self):
        Module(25, ".*Postfix.*", False, 220)

    def test_check_response(self):
        m = Module(25, ".*Postfix.*", False, 220)
        res = m.check_response("220-smtp.server.local ESMTP Postfix (Ubuntu)")
        self.assertTrue(res)


class ExampleTests(unittest.TestCase):
    # def test__software__version__os__version

    def test__postfix__2_11_0__ubuntu__14_04(self):
        m = Module(25, ".*Postfix.*", False, 220)
        res = m.check_response("220-smtp.server.local ESMTP Postfix (Ubuntu)")
        self.assertTrue(res)
