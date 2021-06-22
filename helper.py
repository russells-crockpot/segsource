#!/usr/bin/python3
# pylint: disable=missing-docstring

import pathlib

import click
import toml
from semver import VersionInfo

@click.group()
def cli():
    pass

def for_each_crate_cargo(func):
    cargo_files = [
        pathlib.Path(m) / 'Cargo.toml'
        for m in toml.load('Cargo.toml')['workspace']['members']
    ]
    for path in cargo_files:
        data = toml.load(path)
        res = func(data)
        if res:
            data = res
        with open(path, 'w') as f:
            toml.dump(data, f)

@cli.command('bump-versions')
@click.argument('to-bump',
        type=click.Choice(['major', 'minor', 'patch', 'build', 'prerelease']))
def bump_versions(to_bump):
    def _to_apply(data):
        version = VersionInfo.parse(data['package']['version'])
        if to_bump == 'major':
            version = version.bump_major()
        elif to_bump == 'minor':
            version = version.bump_minor()
        elif to_bump == 'patch':
            version = version.bump_patch()
        elif to_bump == 'build':
            version = version.bump_build()
        elif to_bump == 'prerelease':
            version = version.bump_prerelease()
        data['package']['version'] = str(version)
    for_each_crate_cargo(_to_apply)

@cli.command('set-versions')
@click.argument('to-set', type=click.Choice(['major', 'minor', 'patch', 'build']))
@click.argument('value')
def set_versions(to_set, value):
    def _to_apply(data):
        version = VersionInfo.parse(data['package']['version'])
        if to_set == 'major':
            version = version.replace(major=value)
        elif to_set == 'minor':
            version = version.replace(minor=value)
        elif to_set == 'patch':
            version = version.replace(patch=value)
        elif to_set == 'build':
            version = version.replace(build=value)
        elif to_set == 'prerelease':
            version = version.replace(prerelease=value)
        data['package']['version'] = str(version)
    for_each_crate_cargo(_to_apply)

if __name__ == '__main__':
    cli()
