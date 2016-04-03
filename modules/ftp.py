import text_tcp
from duck import *

import re

class Module(text_tcp.Module):

    _module_protocol = "text/tcp"

    @classmethod
    def check_args(cls, port, response_code):
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        try:
            n = int(response_code)
            if n < 1 or n > 699: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('response_code', response_code, "is not a vaild FTPresponse code")

        return True

    def __init__(self, port, response_code):
        self.port = port
        self.response_code = int(response_code)

    def check_response(self, response):
        try:
            response_code_str = response.split(" ")[0]
            response_code = int(response_code_str)
            if response_code != self.response_code: raise ResponeCheckError("Unexpected response code '%d'; expected '%d'." % (response_code, self.response_code))
        except ValueError:
            raise ResponeCheckError("Invalid response code '%d' in repsonse." % response_code)

        return True

