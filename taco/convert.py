import csv
from typing import List


INTEGER_TYPE = 0
FLOAT_TYPE = 1
VARCHAR_TYPE = 2
TEXT_TYPE = 3

class TypeConversion:
    def __init__(self):
        self.integers = {}
        self.strings = {}
        self.floats = {}

    def convert_integer(self, i: int):
        if i not in self.integers:
            self.integers[i] = len(self.integers) + 1  # + 1, since FROSTT spare tensors are one-indexed
        return self.integers[i]

    def convert_string(self, s: str):
        if s not in self.strings:
            self.strings[s] = len(self.strings) + 1
        return self.strings[s]

    def convert_float(self, f: float):
        if f not in self.floats:
            self.floats[f] = len(self.floats) + 1
        return self.floats[f]


def convert_csv(csv_file_name: str, header_types: List[int], output_file_name: str, has_unique_rows: bool=True):
    tc = TypeConversion()

    with open(csv_file_name, mode='r') as csv_file:
        reader = csv.reader(csv_file, delimiter=',')
        with open(output_file_name, mode='w') as output_file:
            for row in reader:
                output_row = []
                for value, value_type in zip(row, header_types):
                    if value_type == INTEGER_TYPE:
                        output_value = tc.convert_integer(int(value))
                    elif value_type == FLOAT_TYPE:
                        output_value = tc.convert_string(float(value))
                    elif value_type == VARCHAR_TYPE:
                        output_value = tc.convert_string(value)
                    else:
                        # TEXT_TYPE values contain surronding quotations, so we remove it
                        output_value =  tc.convert_string(value[1:-1])
                    output_row.append(output_value)
                if has_unique_rows:
                    output_row.append(1.0)
                output_file.write(' '.join(str(v) for v in output_row))
                output_file.write('\n')

aka_name_header_types = [INTEGER_TYPE, INTEGER_TYPE, TEXT_TYPE, VARCHAR_TYPE, VARCHAR_TYPE, VARCHAR_TYPE, VARCHAR_TYPE, VARCHAR_TYPE]

convert_csv('aka_name.csv', aka_name_header_types, 'aka_name.tns', True)