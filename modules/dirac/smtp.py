from dirac import *
from dirac import text_tcp

import re


class Module(text_tcp.Module):
    @classmethod
    def check_args(cls, port, software, proxy, return_code):
        is_valid_port_number(port)
        is_valid_regex(software, "software")
        try:
            bool(proxy)
        except ValueError:
            raise InvalidArgumentError('proxy', proxy, "is not a bool")
        is_valid_number(return_code, 100, 600, "return_code", "is not a valid return code")

        return True

    def __init__(self, port, software, proxy, return_code):
        self.port = int(port)
        self.software = re.compile(software)
        self.proxy = bool(proxy)
        self.return_code = int(return_code)

    def challenge(self):
        challenge_str = ""
        if self.proxy:
            challenge_str += "PROXY TCP4 127.0.0.1 127.0.0.1 63322 25\n"
        challenge_str += "quit\n"
        return challenge_str

    def check_response(self, response):
        try:
            return_code = int(re.split('-| ', response)[0])
            if self.return_code != return_code:
                raise ResponseCheckError(
                    "Unexpected result code '%d'; expected '%d'." % (return_code, self.return_code))
            if self.software.match(response) is None:
                raise ResponseCheckError(
                    "Unexpected software version '%s'; expected to match against '%s'." % (response, self.software))
        except ValueError:
            raise ResponseCheckError("Invalid identification string '%s' in response." % response)

        return True
