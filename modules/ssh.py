import text_tcp
from duck import *

import re

class Module(text_tcp.Module):

    @classmethod
    def check_args(cls, port, version, software):
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        try:
            re.compile(software)
        except re.error:
            raise InvalidArgumentError('software', software, "is not a valid regular expression")

        return True

    def __init__(self, port, version, software):
        self.port = port
        self.version = version
        self.software = re.compile(software)

    def check_response(self, response):
        try:
            # cf. https://tools.ietf.org/html/rfc4253#section-4.2
            (ssh, version, software) = response.strip().split(' ')[0].split('-', 2)
            if ssh != "SSH": raise ResponeCheckError("Invalid prefix '%s' in repsonse; cf. RFC4253, section 4.2." % ssh)
            if version != self.version: raise ResponeCheckError("Unexpected version '%s'; expected '%s'." % (version, self.version))
            if self.software.match(software) is None: raise ResponeCheckError("Unexpected software version '%s'; expected to match against '%s'." % (software, self.software))
        except ValueError:
            raise ResponeCheckError("Invalid identification string '%s' in repsonse; cf. RFC4253, section 4.2." % response)
        except Exception as err:
            raise ResponeCheckError(err)

        return True

