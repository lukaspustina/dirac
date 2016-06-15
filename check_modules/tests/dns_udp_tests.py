import unittest

from dirac.dns_udp import Module


class UnitTests(unittest.TestCase):
    def test_check_args(self):
        res = Module.check_args(53)
        self.assertTrue(res)

    # noinspection PyMethodMayBeStatic
    def test_init(self):
        Module(53)

    def test_check_response(self):
        m = Module(53)
        res = m.check_response(None)
        self.assertTrue(res)


class ExampleTests(unittest.TestCase):
    # def test__software__version__os__version

    def test__dnsmasq__2_68__ubuntu__14_04(self):
        m = Module(53)
        res = m.check_response(None)
        self.assertTrue(res)
