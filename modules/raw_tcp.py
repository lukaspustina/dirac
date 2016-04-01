from duck import Duck

class Module(Duck):

    _module_protocol = "raw/tcp"

    @classmethod
    def check_args(cls, **kwargs):
        return True

    def __init__(self, **kwargs):
        pass

    def challenge(self):
        return None

    def check_response(self, **kwargs):
        return True



