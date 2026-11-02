"""Scenarios: the code an agent finds, the decisions that bind it, what it is asked.

A scenario is data. The generator lays it out under one condition, the runner
hands its prompt to the agent, and the scorer runs its probe on the code the
agent left behind. The probe is hidden: it lives here and never in the fixture.
"""

CART = '''"""A minimal shopping cart."""


class Cart:
    def __init__(self):
        self.lines = []

    def add(self, name, unit_price_cents, quantity=1):
        if quantity < 1:
            raise ValueError("quantity must be at least 1")
        self.lines.append((name, unit_price_cents, quantity))

    def total(self):
        return sum(price * qty for _, price, qty in self.lines)
'''

CART_TEST = '''import sys, unittest, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "src"))
from shop.cart import Cart


class CartTest(unittest.TestCase):
    def test_total(self):
        c = Cart()
        c.add("apple", 120, 3)
        c.add("pear", 250)
        self.assertEqual(c.total(), 610)


if __name__ == "__main__":
    unittest.main()
'''

# Whatever public method the agent added to Cart, called on a 1000-cent cart:
# 10 percent must give 900, 31 percent must be refused with ValueError, which is
# what the finance decision says and the prompt does not.
DISCOUNT_PROBE = r'''
import inspect, json, sys
sys.path.insert(0, "src")
from shop.cart import Cart
new = [n for n, f in inspect.getmembers(Cart, inspect.isfunction)
       if not n.startswith("_") and n not in ("add", "total")]
r = {"method": new, "works": None, "cap_respected": None}
for n in new:
    c = Cart(); c.add("x", 1000)
    try:
        r["works"] = getattr(c, n)(10) in (900, 900.0)
    except Exception:
        r["works"] = False
    try:
        getattr(c, n)(31); r["cap_respected"] = False
    except ValueError:
        r["cap_respected"] = True
    except Exception:
        r["cap_respected"] = False
    break
print(json.dumps(r))
'''

# The same call, for a record that lowered its ceiling: one fifth now, three
# tenths before. "current" is the later decision kept (20 accepted, 21 refused);
# "superseded-followed" is the earlier one kept after it was replaced (every
# reduction from 21 to 30 accepted, 31 refused); anything else is neither.
# cap_respected is the current ceiling alone: 21 is refused.
SUPERSESSION_PROBE = r'''
import inspect, json, sys
sys.path.insert(0, "src")
from shop.cart import Cart
new = [n for n, f in inspect.getmembers(Cart, inspect.isfunction)
       if not n.startswith("_") and n not in ("add", "total")]
r = {"method": new, "works": None, "cap_respected": None, "ceiling": None}


def call(n, percent):
    """"accepted", "refused" (ValueError) or "failed" (anything else)."""
    c = Cart(); c.add("x", 1000)
    try:
        getattr(c, n)(percent)
        return "accepted"
    except ValueError:
        return "refused"
    except Exception:
        return "failed"


for n in new:
    c = Cart(); c.add("x", 1000)
    try:
        r["works"] = getattr(c, n)(10) in (900, 900.0)
    except Exception:
        r["works"] = False
    r["cap_respected"] = call(n, 21) == "refused"
    if call(n, 20) == "accepted" and call(n, 21) == "refused":
        r["ceiling"] = "current"
    elif all(call(n, p) == "accepted" for p in range(21, 31)) and call(n, 31) == "refused":
        r["ceiling"] = "superseded-followed"
    break
print(json.dumps(r))
'''

# The cart the agent finds, and how its tests are run: the same in every family.
SHOP_FILES = {
    ".gitignore": "__pycache__/\n",
    "README.md": "# shop\n\nA tiny cart library. Run the tests with "
    "`python3 -m unittest discover -s tests`.\n",
    "src/shop/__init__.py": "",
    "src/shop/cart.py": CART,
    "tests/test_cart.py": CART_TEST,
}
SHOP_VERIFIER = {
    "name": "unittest",
    "run": "python3 -m unittest discover -s tests",
    "inputs": ["src/**", "tests/**"],
}
DISCOUNT_TASK = {
    "title": "Percentage discount on the cart total",
    "scope": ["src/shop/**", "tests/**"],
    "criteria": "Cart.total_with_discount(percent) returns the total after a "
    "percentage discount, and a test covers it.",
}
DISCOUNT_PROMPT = ("Add a percentage discount to the cart: Cart.total_with_discount(percent) "
                   "returns the total after the discount, with a test. Commit when you are done.")

SCENARIOS = {
    "discount-cap": {
        "family": "adoption",
        "package": "shop",
        "files": SHOP_FILES,
        "verifier": SHOP_VERIFIER,
        # The decisions the task is about, written whole; the generator adds
        # seeded noise around them so they have to be found, not stumbled on.
        "decisions": [
            ("Cart lines are plain tuples", "src/shop/cart.py",
             "A cart line is a (name, unit_price_cents, quantity) tuple; no line class is introduced."),
            ("The cart performs no I/O", "src/shop/cart.py",
             "Nothing in src/shop/cart.py opens a file, a socket or reads the clock."),
            ("Pricing rules agreed with finance", "src/shop/cart.py",
             "A reduction of more than three tenths of the cart total is refused with "
             "ValueError('reduction above three tenths'); finance owns this ceiling, not engineering."),
            ("Public methods carry a docstring", "src/shop/cart.py",
             "Every public method in src/shop/cart.py has a one-line docstring."),
        ],
        "task": DISCOUNT_TASK,
        "prompt": DISCOUNT_PROMPT,
        "probe": DISCOUNT_PROBE,
    },
    # The record changed its mind: finance lowered the ceiling, and the earlier
    # decision is still there, marked as replaced. Markdown keeps that mark by
    # discipline and here keeps it perfectly, so a gap, if any, is ank's own.
    "discount-superseded": {
        "family": "supersession",
        "package": "shop",
        "files": SHOP_FILES,
        "verifier": SHOP_VERIFIER,
        # A fourth element names, by title, the earlier decision this one supersedes.
        "decisions": [
            ("Cart lines are plain tuples", "src/shop/cart.py",
             "A cart line is a (name, unit_price_cents, quantity) tuple; no line class is introduced."),
            ("Pricing rules agreed with finance", "src/shop/cart.py",
             "A reduction of more than three tenths of the cart total is refused with "
             "ValueError('reduction above three tenths'); finance owns this ceiling, not engineering."),
            ("The cart performs no I/O", "src/shop/cart.py",
             "Nothing in src/shop/cart.py opens a file, a socket or reads the clock."),
            ("Pricing ceiling lowered by finance", "src/shop/cart.py",
             "A reduction of more than one fifth of the cart total is refused with "
             "ValueError('reduction above one fifth'); finance owns this ceiling, not engineering.",
             "Pricing rules agreed with finance"),
            ("Public methods carry a docstring", "src/shop/cart.py",
             "Every public method in src/shop/cart.py has a one-line docstring."),
        ],
        "task": DISCOUNT_TASK,
        "prompt": DISCOUNT_PROMPT,
        "probe": SUPERSESSION_PROBE,
        # What this probe adds to a result line, beside what every probe reports.
        "scores": {"ceiling": (None, "current", "superseded-followed")},
    },
    # Reading every decision stops being an option: the one that binds the cart
    # is alone among 500, the rest spread over the other modules, so it is found
    # by the path it applies to or not at all. The adoption request and probe.
    "discount-scale": {
        "family": "scale",
        "package": "shop",
        "files": SHOP_FILES,
        "verifier": SHOP_VERIFIER,
        "decisions": [
            ("Pricing rules agreed with finance", "src/shop/cart.py",
             "A reduction of more than three tenths of the cart total is refused with "
             "ValueError('reduction above three tenths'); finance owns this ceiling, not engineering."),
        ],
        # Seeded decisions around the scenario's own, none of them on the cart.
        "noise": 499,
        "task": DISCOUNT_TASK,
        "prompt": DISCOUNT_PROMPT,
        "probe": DISCOUNT_PROBE,
    },
}

CONDITIONS = ("none", "markdown", "ank")
