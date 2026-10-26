"""A minimal shopping cart."""


class Cart:
    def __init__(self):
        self.lines = []

    def add(self, name, unit_price_cents, quantity=1):
        if quantity < 1:
            raise ValueError("quantity must be at least 1")
        self.lines.append((name, unit_price_cents, quantity))

    def total(self):
        return sum(price * qty for _, price, qty in self.lines)

    def total_with_discount(self, percent):
        """The total after a percentage discount, refused above three tenths."""
        if percent > 30:
            raise ValueError("reduction above three tenths")
        return self.total() * (100 - percent) // 100
