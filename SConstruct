import os

dirs = [
    "boot",
    "kernel",
]

args = Variables()
args.Add(EnumVariable("build", "Set build type", "debug", allowed_values=["debug", "release"]))
args.Add(BoolVariable("framepointers", "Force frame pointers for stack unwinding in panics", 1))

env = Environment(ENV=os.environ, variables=args)
Export("env")

for d in dirs:
    build_dir = os.path.join("build", d)
    SConscript(os.path.join(d, "SConscript"), variant_dir=build_dir, duplicate=0)

Import("kernel_lib", "boot_objs")

env["LINKFLAGS"] = ["-T", "linker.ld", "-n", "-g"]
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
Default(iso)  # only build this by default
Clean(iso, "build")  # delete whole build dir on clean

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


# really crappy host testing
def host_tests(target, source, env):
    modules = ["kernel/utils"]

    # im so sick of trying to fit discovery and running of tests into scons' sick model that im doing it all right here
    # right now, fk it

    import subprocess
    for module in modules:
        subprocess.run(
            ["cargo", "test", "-Zbuild-std", "--manifest-path", "Cargo.toml", "--features", "std", "--target",
             "x86_64-unknown-linux-gnu"], cwd=module, check=True)

    return []


tests_host = env.Command("test", [], action=host_tests)
env.AlwaysBuild(tests_host)
