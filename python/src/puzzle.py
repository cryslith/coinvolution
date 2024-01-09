from enum import Enum


Display = Enum('Display', ['text', 'line', 'surface'])

class Layer:
    def __init__(self, name, alphas, data, display=None):
        self.name = name
        self.alphas = alphas
        self.data = data
        if display is not None and not isinstance(display, Display):
            raise ValueError
        self.display = display
