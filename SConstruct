import os

dirs = [
    "boot",
    "kernel",
]

out_dir = Dir("#build/")
Export("out_dir")

for d in dirs:
    build_dir = os.path.join("build", d)
    SConscript(os.path.join(d, "SConscript"), variant_dir=build_dir, duplicate=0)

Import("kernel_lib", "boot_objs")

env = Environment(LINKFLAGS=["-T", "linker.ld", "-n", "-g"], ENV=os.environ)
env["CC"] = "ld"  # awful but this has taken long enough

link_binary = env.Program("build/iso/boot/DomeOS", boot_objs, LIBS=[kernel_lib])

# create grub structure and make iso
def mk_grub(env, target, source):
    os.makedirs("build/iso/boot/grub", exist_ok=True)
    grub = \
        """set timeout=0
        set default=0
        menuentry "domeos" {
            multiboot /boot/DomeOS some args in grub.cfg
            boot
        }"""

    with open("build/iso/boot/grub/grub.cfg", "w") as f:
        f.write(grub)

    env.Execute("grub-mkrescue -o build/DomeOS.iso build/iso")


iso = env.Command("build/DomeOS.iso", [link_binary, "build/boot"], action=mk_grub)
env.Depends(iso, link_binary)
Default(iso) # only build this by default
Clean(iso, "build") # delete whole build dir on clean

# run command
qemu_cmd = "qemu-system-x86_64 -cdrom build/DomeOS.iso -monitor stdio -serial file:serial.log -d cpu_reset,int -D qemu-logfile -no-reboot"
if "debug" in COMMAND_LINE_TARGETS:
    qemu_cmd += " -s -S"

def PhonyTarget(target, action):
    env = Environment(ENV=os.environ, BUILDERS={"phony": Builder(action=action)})
    phony = env.phony(target=target, source="SConstruct")
    AlwaysBuild(phony)
    Requires(phony, iso)


run_qemu = PhonyTarget(["run", "debug"], qemu_cmd)
env.Depends(run_qemu, iso)
