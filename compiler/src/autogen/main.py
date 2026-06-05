from gen import *
import x86
import lir
import argparse


class Args:
    target: str
    pass


def main(args: Args):
    match args.target:
        case "x86":
            print(x86.iclass.gen_file())
            return
        case "lir":
            print(lir.iclass.gen_file())
            return
        case "riscv":
            pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        "igen",
        description="Tool to generate Rust files containing enum definitions and associated methods for a set of pre-defined instructions.",
    )
    parser.add_argument("-t", "--target", required=True, choices=["lir", "x86", "riscv"])
    args = Args()
    parser.parse_args(namespace=Args)
    main(args)
