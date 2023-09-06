import json
import pandas as pd
import os

columns = ['productid', 'productname','elementid', 'elementname', 'enddate','discnttype', 'discntvalue', 'startdate','elementtype', 'discntcode', 'discntname', 'spproductid', 'spproductname']


def extract_array(data, columns):
    rows = []
    for item in data:
        row = {}
        for c in columns:
            value = item.get(c)
            if isinstance(value, dict) or isinstance(value, list):
                value = json.dumps(value)
            row[c] = value
        rows.append(row)
        # row['productname'] = item.get('productname')
    # rows.append({'': 'xxx', '': 'xxx'})
    return rows


def extract_rows(data, columns):
    rows = []

    rows1 = extract_array(data['otherinfo'], columns)
    rows2 = extract_array(data['flowpackageinfo'], columns)

    for item in data['productinfo']:
        row = {}
        for c in columns:
            if c in item:
                row[c] = item[c]
        rows.append(row)

    return rows1 + rows2 + rows


with open('Test.json') as f:
    data = json.load(f)
cwd = os.getcwd()
from collections import OrderedDict

data = OrderedDict(data)
rows1 = extract_array(data['otherinfo'], columns)
# rows1 = extract_rows(data['otherinfo'], columns)
rows2 = extract_array(data['flowpackageinfo'], columns)
rows3 = extract_array(data['spinfo'], columns)
rows4 = extract_array(data['productinfo'], columns)
rows = rows1 + rows2 + rows3 + rows4
# rows = extract_rows(data, columns)

df = pd.DataFrame(rows, columns=columns, dtype=str)
excel_file = os.path.join(cwd, 'data.xlsx')
df.to_excel(excel_file, index=False)
# print(data['spinfo'])
