import text_tcp
from duck import *

import re

class Module(text_tcp.Module):

    @classmethod
    def check_args(cls, **kwargs):
        if not 'port' in kwargs: raise MissingArgumentError('port')
        port = kwargs['port']
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        if not 'version' in kwargs: raise MissingArgumentError('version')

        if not 'software' in kwargs: raise MissingArgumentError('software')
        software = kwargs['software']
        try:
            re.compile(software)
        except re.error:
            raise InvalidArgumentError('software', software, "is not a valid regular expression")

        return True

    def __init__(self, **kwargs):
        self.port = kwargs['port']
        self.version = kwargs['version']
        self.software = re.compile(kwargs['software'])

    def check_response(self, **kwargs):
        if not 'response' in kwargs: raise MissingArgumentError('response')
        response = kwargs['response']

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

