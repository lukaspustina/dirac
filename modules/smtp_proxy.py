import smtp
from dirac import *

import re

class Module(smtp.Module):

    def challenge(self):
        return "PROXY TCP4 127.0.0.1 127.0.0.1 63322 25\n"

