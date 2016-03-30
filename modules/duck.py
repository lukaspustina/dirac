class Duck(object):

    _module_protocol = ""

    @classmethod
    def protocol(cls):
        return cls._module_protocol

    @classmethod
    def check_args(cls, **kwargs):
        raise NotImplementedError

    def __init__(self, **kwargs):
        pass

    def challenge(self):
        return None

    def check_response(self, **kwargs):
        raise NotImplementedError


class Error(Exception):
    """Base class for exceptions in this module."""
    pass


class InvalidArgumentError(Error):

    def __init__(self, argument, value, reason=None):
        self.argument = argument
        self.value = value
        self.reason = reason

    def __str__(self):
        return "%s: %s%s" % (repr(self.argument), repr(self.value), " %s" if reason else "")


class ResponeCheckError(Error):

    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message

