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

SCENARIOS = {
    "discount-cap": {
        "family": "adoption",
        "package": "shop",
        "files": {
            ".gitignore": "__pycache__/\n",
            "README.md": "# shop\n\nA tiny cart library. Run the tests with "
            "`python3 -m unittest discover -s tests`.\n",
            "src/shop/__init__.py": "",
            "src/shop/cart.py": CART,
            "tests/test_cart.py": CART_TEST,
        },
        "verifier": {
            "name": "unittest",
            "run": "python3 -m unittest discover -s tests",
            "inputs": ["src/**", "tests/**"],
        },
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
        "task": {
            "title": "Percentage discount on the cart total",
            "scope": ["src/shop/**", "tests/**"],
            "criteria": "Cart.total_with_discount(percent) returns the total after a "
            "percentage discount, and a test covers it.",
        },
        "prompt": "Add a percentage discount to the cart: Cart.total_with_discount(percent) "
        "returns the total after the discount, with a test. Commit when you are done.",
        "probe": DISCOUNT_PROBE,
    },
}

CONDITIONS = ("none", "markdown", "ank")
