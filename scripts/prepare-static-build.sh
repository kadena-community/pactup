#!/bin/bash
sed -i 's@"flags": \[\]@"flags": ["-ccopt", "-static"]@' package.json
