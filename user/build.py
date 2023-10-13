import os
import re

base_addr_ori = 0x80400000
step = 0x20000
linker = 'src/linker.ld'

base_addr = base_addr_ori
apps = os.listdir('src/bin')
apps.sort()
suff = re.compile(r'(.*)\.rs$')
for app in apps:
    app_name = suff.match(app).group(1)
    lines = []
    lines_ori = []
    with open(linker, 'r') as f:
        for line in f.readlines():
            lines_ori.append(line)
            line = line.replace(hex(base_addr_ori), hex(base_addr))
            lines.append(line)
    with open(linker, 'w+') as f:
        f.writelines(lines)
    os.system(f'cargo build --bin {app_name} --release')
    print(f'[builder] set base address of {app_name} as {hex(base_addr)}. ')
    with open(linker, 'w+') as f:
        f.writelines(lines_ori)
    base_addr += step