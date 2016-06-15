import re


class Module(object):
    _module_protocol = ""

    @classmethod
    def protocol(cls):
        return cls._module_protocol

    @classmethod
    def check_args(cls, **kwargs):
        raise NotImplementedError

    def __init__(self, **kwargs):
        pass

    # noinspection PyMethodMayBeStatic
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
        return "%s: %s%s" % (repr(self.argument), repr(self.value), " %s" if self.reason else "")


class ResponseCheckError(Error):
    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message


def is_valid_number(number, minimum, maximum, name="number", msg="is not a valid number"):
    try:
        n = int(number)
        if n < minimum or n > maximum:
            raise ValueError()
    except ValueError:
        raise InvalidArgumentError(name, number, msg)


def is_valid_port_number(port):
    is_valid_number(port, 1, 0xFFFF, "port", "is not valid port number")


def is_valid_regex(regex, name, msg="is not a valid regular expression"):
    try:
        re.compile(regex)
    except re.error:
        raise InvalidArgumentError(name, regex, msg)

    return True
