class LogHelper:
    def __init__(self, name):
        self.name = name

    ANSI_C_RED = '31'
    ANSI_C_GRN = '32'
    ANSI_C_YLW = '33'
    ANSI_C_BLU = '34'
    ANSI_C_MAG = '35'
    ANSI_C_CYN = '36'
    ANSI_C_WHT = '37'

    @staticmethod
    def cprint(c, s):
        print(f"\033[{c}m{s}\033[0m")