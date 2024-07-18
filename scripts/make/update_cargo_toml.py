import os
import re
from enum import Enum, auto
from collections import defaultdict

class Status(Enum):
    BEFORE = auto()
    RUNNING = auto()
    AFTER = auto()

def update_cargo_toml(cargo_toml_path, apps_src, mode):
    std_apps = parse_std_apps(apps_src) if mode == 'update' else []
    lines_before_member, lines_after_member = parse_cargo_toml(cargo_toml_path)

    with open(cargo_toml_path, 'w') as new_cargo_toml:
        for line in lines_before_member:
            new_cargo_toml.write(line)
        for std_app in std_apps:
            new_cargo_toml.write('    "%s",\n'%(std_app))
        for line in lines_after_member:
            if not line.startswith('toml_edit = '):
                new_cargo_toml.write(line)
        if mode == 'update':
            new_cargo_toml.write('toml_edit = { path = "crates/toml/crates/toml_edit" }')

def parse_std_apps(apps_src):
    ignore_dirs = ['.git']

    prefix = 'apps/std'
    std_apps = []
    for app in os.listdir(apps_src):
        app_src = os.path.join(apps_src, app)
        if app in ignore_dirs or not os.path.isdir(app_src):
            continue
        sub_directories = os.listdir(app_src)
        if 'Cargo.toml' in sub_directories and 'src' in sub_directories:
            std_apps.append(os.path.join(prefix, app))
        else:
            for sub_app in sub_directories:
                std_apps.append(os.path.join(prefix, app, sub_app))

    return std_apps

def parse_cargo_toml(cargo_toml_path):
    lines_before_member = []
    lines_after_member = []
    with open(cargo_toml_path, 'r') as file:
        lines = file.readlines()
        status = Status.BEFORE
        for line in lines:
            if status == Status.BEFORE:
                lines_before_member.append(line)
                if line.strip().startswith('members'):
                    status = Status.RUNNING
            elif status == Status.RUNNING:
                if line.strip().startswith(']'):
                    lines_after_member.append(line)
                    status = Status.AFTER
                elif not line.strip().startswith('"apps/std'):
                    lines_before_member.append(line)
            else:
                lines_after_member.append(line)

    return lines_before_member, lines_after_member



if __name__ == "__main__":
    import sys
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--cargo_toml_path', type=str, required=True, help='Please input Cargo.toml path')
    parser.add_argument('--apps_src_path', type=str, required=True, help='Please input std_apps path')
    parser.add_argument('--mode', type=str, default='update', help='Please input the mode: update/delete')

    args = parser.parse_args()

    update_cargo_toml(args.cargo_toml_path, args.apps_src_path, args.mode)
