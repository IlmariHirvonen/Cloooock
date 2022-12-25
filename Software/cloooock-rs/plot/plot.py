import matplotlib.pyplot as plt

with open('encoder.txt', 'r') as fd:
    lines = fd.readlines()

    clk = []
    dt = []
    for line in lines:
        vals = line.split(' ')
        clk.append(int(vals[0]))
        dt.append(int(vals[1]))
    
    plt.figure()
    plt.plot(clk, color='yellow')
    plt.plot(dt, color='blue')
    plt.show()
