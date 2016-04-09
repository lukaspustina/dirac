import text_tcp
from dirac import *

import re

class Module(text_tcp.Module):

    @classmethod
    def check_args(cls, port, software, return_code):
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        try:
            re.compile(software)
        except re.error:
            raise InvalidArgumentError('software', software, "is not a valid regular expression")

        try:
            n = int(return_code)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('return_code', return_code, "is not a vaild port number")

        return True

    def __init__(self, port, software, return_code):
        self.port = int(port)
        self.software = re.compile(software)
        self.return_code = int(return_code)

    def challenge(self):
        return "quit"

    def check_response(self, response):
        try:
            return_code = int(re.split('-| ', response)[0])
            if self.return_code != return_code: raise ResponeCheckError("Unexpected result code '%d'; expected '%d'." % (return_code, self.return_code))
            if self.software.match(response) is None: raise ResponeCheckError("Unexpected software version '%s'; expected to match against '%s'." % (response, self.software))
        except ValueError:
            raise ResponeCheckError("Invalid identification string '%s' in repsonse." % response)

        return True

