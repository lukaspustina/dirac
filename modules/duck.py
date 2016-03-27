class Duck(object):
    _module_protocol = ""

    @classmethod
    def protocol(cls):
        return cls._module_protocol

    @classmethod
    def check_args(cls, **kwargs):
        raise NotImplementedError

    def __init__(self):
        pass

    def challenge(self):
        return None

    def check_response(self):
        raise NotImplementedError

