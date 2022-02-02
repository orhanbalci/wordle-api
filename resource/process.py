source = open('words.txt', 'r')
lines = source.readlines()
fives = []
for line in lines:
    line = line.strip().lower()
    if len(line) == 5 and 'â' not in line:
        print(f'adding {line}')
        fives.append(f"\"{line}\",\n")

destination = open('fives.txt', 'w')
destination.write('{ words = [')
destination.writelines(fives)
destination.write(']}')
destination.close()
source.close()
