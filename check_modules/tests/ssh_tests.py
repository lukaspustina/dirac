import unittest

from dirac.ssh import Module


class UnitTests(unittest.TestCase):
    def test_check_args(self):
        res = Module.check_args(22, "2.0", "OpenSSH.*")
        self.assertTrue(res)

    # noinspection PyMethodMayBeStatic
    def test_init(self):
        Module(22, "2.0", "OpenSSH.*")

    def test_check_response(self):
        m = Module(22, "2.0", "OpenSSH.*")
        res = m.check_response("SSH-2.0-OpenSSH_6.6.1p1 Ubuntu-2ubuntu2")
        self.assertTrue(res)


class ExampleTests(unittest.TestCase):
    # def test__software__version__os__version

    def test__openssh__6_6__ubuntu__14_04(self):
        m = Module(22, "2.0", "OpenSSH.*")
        res = m.check_response("SSH-2.0-OpenSSH_6.6.1p1 Ubuntu-2ubuntu2")
        self.assertTrue(res)
