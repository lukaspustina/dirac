from dirac import *

class Module(Dirac):
    _module_protocol = "raw/tcp"

    @classmethod
    def check_args(cls, port):
        is_valid_port_number(port)
        return True

    def __init__(self, port):
        self.port = port

    def challenge(self):
        return None

    def sanity_check(self, response):
        if response == None:
            return False

        full_response_length = len(response)
        # https://dev.mysql.com/doc/internals/en/mysql-packet.html
        # 4 bytes packet header minimum
        if full_response_length < 4:
        # Response length < 4 -- missing packet header?
            return False

        actual_payload_length = full_response_length - 4

        # 3 bytes payload length announcement
        expected_payload_length = 0
        for i in range(3):
            expected_payload_length += response[i] << (8*i)

        if expected_payload_length == 0:
            # Announced payload length is 0 -- this cannot be right
            return False

        if actual_payload_length != expected_payload_length:
            # Actual Payload Length != Expected Length
            return False

        # 1 byte sequence id -- for payloads > 16MB
        sequence_id = response[3]
        if sequence_id != 0:
            # Rejecting packet sequence id > 1
            return False

        return True


    # https://dev.mysql.com/doc/internals/en/packet-ERR_Packet.html
    def is_error_packet(self, payload):
        payload_length = len(payload)

        if payload_length < 9:
            return False
        ff_header = payload[0]

        if 0xFF != ff_header:
            return False
        return True

    # https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::Handshake
    def is_initial_handshake_packet(self, payload):
        payload_length = len(payload)
        offset = 0

        if payload_length < 15:
            return False

        if payload_length > 128:
            # Simple Sanity Check -- there are variable length strings,
            # but the server version should not be _that_ long
            return False

        protocol_version = payload[offset]
        offset += 1
        if protocol_version != 0x0A:
            return False

        # Read NUL terminated version string
        buf = bytearray()
        for b in payload[offset:]:
            offset += 1
            if b != 0x00:
                buf.append(b)
            else:
                break

        # Not used for now
        server_version = buf.decode("iso-8859-1")

        # There must be three more fields left
        if offset >= payload_length:
            return False

        # Determined by server. Not particularly useful here.
        connection_id = payload[offset : offset + 4]
        offset+=4

        # Determined by server. Not particularly useful here.
        auth_plugin_data_part_1 = payload[offset : offset + 8]
        offset += 8

        # Filler NUL byte, always here.
        filler = payload[offset]
        offset +=1
        if filler != 0x00:
            return False

        # After the filler, all further fields are optional
        return True


    def check_response(self, response):
        try:
            if not self.sanity_check(response):
                return False

            payload = response[4:]

            # MySQL will answer with an error response, if you are
            # not allowed to connect, e. g. from an unauthorized
            # incoming IP address. This confirms the presence of
            # a MysQL Server though :)
            if self.is_error_packet(payload):
                return True

            if self.is_initial_handshake_packet(payload):
                return True

        except ValueError:
            raise ResponseCheckError("Did not get expected response: %s" % response)
        return False

