from duck import Duck

class Module(Duck):

    _module_protocol = "text/udp"

    @classmethod
    def check_args(cls, port):
        try:
            n = int(port)
            if n < 1 or n > 0xFFFF: raise ValueError()
        except ValueError as err:
            raise InvalidArgumentError('port', port, "is not a vaild port number")

        return True

    def __init__(self, port):
        self.port = port

    def challenge(self):
        return "Dirac"

    def check_response(self, response):
        try:
            return len(response) > 0
        except ValueError:
            raise ResponeCheckError("Invalid identification string '%s' in repsonse; cf. RFC4253, section 4.2." % response)

        return True

