# tool.py: the demo module.
# It has a two-line header, then code.

import os

TEMPLATE = """
# This line is in a string, not a comment.
# Neither is this one.
"""

# A real block after the string.
def run():
    return os.getcwd()
