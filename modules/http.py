import text_tcp
from duck import *

import re

class Module(text_tcp.Module):

    _module_protocol = "http/tcp"

    @classmethod
    def check_args(cls, port, verb, uri, response_code):
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        if verb.upper() not in ['GET', ]:
            raise InvalidArgumentError('verb', verb, "is not a vaild HTTP verb")

        if uri == "":
            raise InvalidArgumentError('uri', uri, "is not a vaild URI")

        try:
            n = int(response_code)
            if n < 1 or n > 599: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('response_code', response_code, "is not a vaild HTTP response code")

        return True

    def __init__(self, port, verb, uri, response_code):
        self.port = port
        self.verb = verb.upper()
        self.uri = uri
        self.response_code = int(response_code)

    def challenge(self):
        return "%s %s" % (self.verb, self.uri)

    def check_response(self, response_code, header, body):
        try:
            response_code = int(response_code)
            if response_code != self.response_code: raise ResponeCheckError("Unexpected response code '%d'; expected '%d'." % (response_code, self.response_code))
        except ValueError:
            raise ResponeCheckError("Invalid response code '%d' in repsonse." % response_code)

        return True

