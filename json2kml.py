#!/usr/bin/env python3

import json
import simplekml

with open('./stops.json', 'r') as f:
    stops = json.load(f)['0']

kml = simplekml.Kml()
kml.document.name = "Telegram Locations"

for id in stops.keys():
    kml.newpoint(name = str(id), coords = [( stops[id]['lon'], stops[id]['lat'] )])

kml.save(path = './stops.kml')

