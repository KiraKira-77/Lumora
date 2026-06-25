import sys
from rembg import remove
from PIL import Image

def remove_background(input_path, output_path):
    print(f"Reading {input_path}...")
    with open(input_path, 'rb') as i:
        input_data = i.read()
    print("Removing background...")
    output_data = remove(input_data)
    with open(output_path, 'wb') as o:
        o.write(output_data)
    print(f"Saved to {output_path}")

if __name__ == '__main__':
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    remove_background(input_file, output_file)
