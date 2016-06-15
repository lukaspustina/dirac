from dirac import *
from dirac import text_tcp


class Module(text_tcp.Module):
    _module_protocol = "text/tcp"

    @classmethod
    def check_args(cls, port, response_code):
        is_valid_port_number(port)
        is_valid_number(response_code, 1, 699, "response_code", "is not a valid FTP response code")

        return True

    def __init__(self, port, response_code):
        self.port = port
        self.response_code = int(response_code)

    def check_response(self, response):
        try:
            response_code_str = response.split(" ")[0]
            response_code = int(response_code_str)
            if response_code != self.response_code:
                raise ResponseCheckError(
                    "Unexpected response code '%d'; expected '%d'." %
                    (response_code, self.response_code))
        except ValueError:
            raise ResponseCheckError("Invalid response code '%d' in response." % response)

        return True
