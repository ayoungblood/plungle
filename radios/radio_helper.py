from decimal import Decimal

class RadioHelper:
    def __init__(self, name):
        self.name = name

    @staticmethod
    def mhzStr_to_hzInt(mhz_str):
        mhz_decimal = Decimal(mhz_str)
        hz_decimal = mhz_decimal * Decimal('1000000')
        hz = int(hz_decimal)
        return hz

    @staticmethod
    def hzInt_to_mhzStr(hz):
        hz_decimal = Decimal(hz)
        mhz_decimal = hz_decimal / Decimal('1000000')
        return f"{mhz_decimal:.4f}"

    @staticmethod
    def khzStr_to_hzInt(khz_str):
        khz_decimal = Decimal(khz_str)
        hz_decimal = khz_decimal * Decimal('1000')
        hz = int(hz_decimal)
        return hz

    @staticmethod
    def is_float_str(s):
        if s.count('.') > 1:
            return False
        parts = s.split('.')
        if len(parts) > 2:
            return False
        return all(part.isnumeric() for part in parts if part)

    @staticmethod
    def is_2m(freq):
        return 144.0E6 <= freq <= 148.0E6

    @staticmethod
    def is_70cm(freq):
        return 420.0E6 <= freq <= 450.0E6