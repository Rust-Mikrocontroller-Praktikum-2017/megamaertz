#!/usr/bin/python

import os
import sys

# pip install Pillow
from PIL import Image


if __name__ == 'main':
    if len(sys.argv) < 2:
        print "ERROR: no file provided"
        exit(404)

    PATH_STR = sys.argv[1]
    if not (os.path.exists(PATH_STR) and os.path.isfile(PATH_STR)):
        print "ERROR: Provided argument is not a vaid file"
        exit(403)

    IMG = Image.open(PATH_STR)
    with open(PATH_STR[:-3] + "dump", "wb") as out:
        for c in list(IMG.getdata()):
            out.write(chr(c[0]) + chr(c[1]) + chr(c[2]))

    print "Done"
