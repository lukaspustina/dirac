import http
from dirac import *

import re

class Module(http.Module):

    _module_protocol = "https/tcp"

