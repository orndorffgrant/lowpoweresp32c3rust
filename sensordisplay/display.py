import time
import serial
import numpy as np
from live_plotter import FastLivePlotter

live_plotter = FastLivePlotter(titles=["mA"], n_rows=1, n_cols=1)
data = []

SERIAL = serial.Serial("/dev/ttyUSB0", 115200)
SERIAL.reset_input_buffer()
try:
    while True:
        while SERIAL.in_waiting == 0:
            time.sleep(0.03)
        line = SERIAL.readline().strip()
        try:
            data.append(float(line))
        except ValueError:
            continue
        data = data[-300:]
        live_plotter.plot(
            y_data_list=[np.array(data)],
        )
        ave = np.average(data)
        mini = np.min(data)
        maxi = np.max(data)
        padding = "                                                                                         "
        print(f"\rAverage: {ave:6.6}mA\tMin: {mini:6.6}mA\tMax: {maxi:6.6}mA", end="")
except KeyboardInterrupt:
    print("Ctrl-C")
    pass