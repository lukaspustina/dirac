from dirac import *
import dirac


class Module(dirac.Module):
    _module_protocol = "text/udp"

    @classmethod
    def check_args(cls, port):
        is_valid_port_number(port)

        return True

    def __init__(self, port):
        self.port = port

    def challenge(self):
        return "Dirac"

    def check_response(self, response):
        try:
            return not response or len(response) > 0
        except ValueError:
            raise ResponseCheckError(
                "Invalid identification string '%s' in response; cf. RFC4253, section 4.2." %
                response)
