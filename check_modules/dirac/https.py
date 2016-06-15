from dirac import *
from dirac import http


class Module(http.Module):
    _module_protocol = "https/tcp"
