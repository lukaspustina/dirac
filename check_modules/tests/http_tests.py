import unittest

from dirac.http import Module


class UnitTests(unittest.TestCase):
    def test_check_args_get(self):
        res = Module.check_args(80, "GET", "http://server.local/index.html", 200)
        self.assertTrue(res)

    # noinspection PyMethodMayBeStatic
    def test_init_get(self):
        Module(80, "GET", "http://server.local/index.html", 200)

    def test_check_response(self):
        m = Module(80, "GET", "http://server.local/index.html", 200)
        res = m.check_response(200,
                               """
HTTP/1.1 200 OK
Date: Tue, 19 Apr 2016 10:18:08 GMT
Server: Apache/2.4.7 (Ubuntu)
Last-Modified: Tue, 15 Mar 2016 15:06:51 GMT
ETag: "1b4a-52e17bf402ab3"
Accept-Ranges: bytes
Content-Length: 6986
Vary: Accept-Encoding
Content-Type: text/html
""",
                               "")
        self.assertTrue(res)


class ExampleTests(unittest.TestCase):
    # def test__software__version__os__version

    def test__apache__2_4_7__ubuntu__14_04(self):
        m = Module(80, "GET", "http://server.local/index.html", 200)
        res = m.check_response(200,
                               """
HTTP/1.1 200 OK
Date: Tue, 19 Apr 2016 10:18:08 GMT
Server: Apache/2.4.7 (Ubuntu)
Last-Modified: Tue, 15 Mar 2016 15:06:51 GMT
ETag: "1b4a-52e17bf402ab3"
Accept-Ranges: bytes
Content-Length: 6986
Vary: Accept-Encoding
Content-Type: text/html
""",
                               "")
        self.assertTrue(res)
