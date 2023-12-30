#!/usr/bin/env python3

from app import create_app
import simpleloop

if __name__ == '__main__':
    create_app({'custom': simpleloop.solve}).run()
