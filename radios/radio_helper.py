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

    def hzInt_to_mhzStr(hz):
        hz_decimal = Decimal(hz)
        mhz_decimal = hz_decimal / Decimal('1000000')
        return f"{mhz_decimal:.4f}"

    @staticmethod
    def is_2m(freq):
        return 144.0E6 <= freq <= 148.0E6

    @staticmethod
    def is_70cm(freq):
        return 420.0E6 <= freq <= 450.0E6