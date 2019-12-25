def fuel(mass):
    return mass // 3 - 2

def extra_fuel(mass):
    mass = fuel(mass)
    if mass <= 0:
        return 0
    return mass + extra_fuel(mass)

with open("input") as f:
    print("1-1:")
    print(sum([fuel(int(line)) for line in f.readlines()]))
with open("input") as f:
    print("1-2:")
    print(sum([extra_fuel(int(line)) for line in f.readlines()]))
