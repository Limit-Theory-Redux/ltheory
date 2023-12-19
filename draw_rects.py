import matplotlib.pyplot as plt
import numpy as np
import argparse

# Function to parse the file and extract rectangles with colors
def parse_rectangles(file_path):
    rectangles = []
    colors = {'Group': 'red', 'Rect': 'blue', 'Image': 'green', 'Text': 'magenta'}  # Define colors for each rectangle type
    x_coords, y_coords = [], []

    with open(file_path, 'r') as file:
        for line in file:
            parts = line.strip().split()
            rect_type, x, y, width, height = parts[0], float(parts[1]), float(parts[2]), float(parts[3]), float(parts[4])
            rectangles.append((rect_type, x, y, width, height, colors.get(rect_type, 'black'))
            )
            x_coords.extend([x, x + width])
            y_coords.extend([y, y + height])

    padding = 10

    xlim = [min(x_coords) - padding, max(x_coords) + padding]
    ylim = [min(y_coords) - padding, max(y_coords) + padding]

    return rectangles, xlim, ylim

# Function to visualize rectangles
def visualize_rectangles(rectangles, xlim, ylim):
    plt.figure()
    for rect_type, x, y, width, height, color in rectangles:
        plt.gca().add_patch(plt.Rectangle((x, y), width, height, fill=None, edgecolor=color, label=rect_type))
    plt.xlim(xlim[0], xlim[1])
    plt.ylim(ylim[0], ylim[1])
    plt.gca().set_aspect('equal', adjustable='box')
    plt.legend()
    plt.show()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Visualize rectangles from a file.')
    parser.add_argument('file_name', help='Name of the input file containing rectangle data')
    args = parser.parse_args()

    rectangles, xlim, ylim = parse_rectangles(args.file_name)
    visualize_rectangles(rectangles, xlim, ylim)
