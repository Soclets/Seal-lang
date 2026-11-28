# trace: D155, D159. The exact interest of `interest.normal.wax` in Python's `decimal`, with room for every digit, so
# nothing is ever rounded (an inexact step would stop it).
import decimal
from decimal import Decimal
decimal.setcontext(decimal.Context(prec=decimal.MAX_PREC, Emax=decimal.MAX_EMAX, Emin=decimal.MIN_EMIN, traps=[decimal.Inexact, decimal.Overflow, decimal.InvalidOperation]))
RATE = Decimal("1.05")
def run(n):
    savings = Decimal("100.00")
    for _ in range(n):
        savings = savings * RATE
    return len(str(savings))
