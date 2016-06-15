from dirac import *
from dirac import text_tcp

import re


class Module(text_tcp.Module):
    _module_protocol = "http/tcp"

    @classmethod
    def check_args(cls, port, verb, uri, response_code):
        is_valid_port_number(port)
        Module._is_valid_http_verb(verb)
        Module._is_valid_uri(uri)
        is_valid_number(response_code, 1, 599, "response_code", "is not a valid HTTP response code")

        return True

    @classmethod
    def _is_valid_http_verb(cls, verb):
        if verb.upper() not in ['GET', ]:
            raise InvalidArgumentError('verb', verb, "is not a valid HTTP verb")

    @classmethod
    def _is_valid_uri(cls, uri):
        if uri == "":
            raise InvalidArgumentError('uri', uri, "is not a valid URI")

    def __init__(self, port, verb, uri, response_code):
        self.port = port
        self.verb = verb.upper()
        self.uri = uri
        self.response_code = int(response_code)

    def challenge(self):
        return "%s %s" % (self.verb, self.uri)

    def check_response(self, response_code, headers, body):
        try:
            response_code = int(response_code)
            if response_code != self.response_code:
                raise ResponseCheckError("Unexpected response code '%d'; expected '%d'." %
                                         (response_code, self.response_code))
        except ValueError:
            raise ResponseCheckError("Invalid response code '%d' in response." % response_code)

        return True
